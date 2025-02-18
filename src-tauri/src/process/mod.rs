mod filter;
mod max;

pub use filter::ImageFilter;

use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Write},
    path::Path,
};

use anyhow::{anyhow, Context, Result};
use image::{ImageBuffer, ImageFormat, ImageReader, Luma};
use ndarray::prelude::*;
use nshare::IntoNdarray2;
use rayon::prelude::*;
use tauri::{async_runtime::Mutex, ipc::Channel, State};

use crate::{
    parse_xml::{ChannelID, Harmony, Image},
    AppState,
};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct OutputInfo {
    pub dir: std::path::PathBuf,
    pub action: String,
    pub format: String,
}

#[derive(serde::Serialize)]
pub struct DownloadInfo {
    name: String,
    rows: u16,
    cols: u16,
    output: OutputInfo,
    filter: ImageFilter,
}

impl TryFrom<&AppState> for DownloadInfo {
    type Error = anyhow::Error;
    fn try_from(state: &AppState) -> Result<Self> {
        let info = state
            .info
            .as_ref()
            .ok_or_else(|| anyhow!("Missing measurement info"))?;
        let output = state
            .output
            .as_ref()
            .ok_or_else(|| anyhow!("Missing output info"))?;
        let filter = state
            .filter
            .as_ref()
            .ok_or_else(|| anyhow!("Missing filter info"))?;

        let rows = info.plate.rows;
        let cols = info.plate.cols;

        Ok(Self {
            name: info.plate.name.clone(),
            rows,
            cols,
            output: output.clone(),
            filter: filter.clone(),
        })
    }
}

type YX = Array2<u16>;
type Int16 = ImageBuffer<Luma<u16>, Vec<u16>>;

/// Helper function to converted 2D NDArray into Image crate Image
fn array_to_image(arr: YX) -> Int16 {
    // image crate wants row order
    // https://stackoverflow.com/questions/56762026/how-to-save-ndarray-in-rust-as-image
    let arr = arr.as_standard_layout().to_owned();
    let (h, w) = arr.dim();
    let (raw, _) = arr.into_raw_vec_and_offset();
    ImageBuffer::from_raw(w as u32, h as u32, raw).unwrap()
}

fn max_project(acc: Option<YX>, img: &Image) -> Result<Option<YX>> {
    let res = reqwest::blocking::get(&img.url).context("getting image")?;
    let raw = res.bytes().context("reading response bytes")?;

    let pixels = ImageReader::with_format(Cursor::new(&raw), ImageFormat::Tiff)
        .decode()
        .context("reading TIFF into pixels")?
        .into_luma16()
        .into_ndarray2();

    let acc = acc
        .map(|mut acc| {
            azip!((a in &mut acc, &b in &pixels) *a = (*a).max(b));
            acc
        })
        .or_else(|| Some(pixels));

    Ok(acc)
}

fn dl_and_maxproj(hm: &Harmony, dir: &Path) -> Result<()> {
    // group (row, col, channel, time, field)
    type Key = (u16, u16, ChannelID, u32, u32);
    type Map<'a> = HashMap<Key, Vec<&'a Image>>;

    let cmap = &hm.channels;

    let init: Map = HashMap::new();

    // for testing, filter to 3 fields in well A1
    let datasets = hm
        .images
        .iter()
        .filter(|img| img.row == 1 && img.col == 1 && [1, 2, 3].contains(&img.field));

    let datasets = datasets.fold(init, |mut acc, img| {
        let key = (img.row, img.col, img.channel, img.timepoint, img.field);

        acc.entry(key)
            .and_modify(|v| v.push(img))
            .or_insert_with(|| vec![img]);

        acc
    });

    datasets
        .into_par_iter()
        .map(|(k, imgs)| {
            imgs.into_iter()
                .try_fold(None, max_project)
                .map(|res| (k, res))
        })
        .try_for_each(|res| {
            let (k, projection) = res.context("projecting image")?;
            let projection = projection
                .ok_or_else(|| anyhow::anyhow!("missing projection (ADD well info...!!)"))?;

            let (r, c, ch, tp, f) = k;
            let img = array_to_image(projection);
            let output = dir.join(format!(
                "{}-R{r:02}C{c:02}F{f:03}T{tp}.tiff",
                cmap[&ch].name
            ));

            img.save_with_format(&output, ImageFormat::Tiff)
                .with_context(|| format!("saving image to {}", output.display()))
        })
}

async fn _download_image(hm: &Harmony, dir: &Path) -> Result<u64> {
    let iter = hm
        .images
        .iter()
        .filter(|&img| img.row == 1 && img.col == 1 && img.timepoint == 1 && img.field == 1);

    let cmap = &hm.channels;
    let mut count = 0;
    for img in iter {
        let f = dir.join(format!(
            "R{:02}C{:02}F{:03}P{:03}-{}.tiff",
            img.row, img.col, img.field, img.plane, cmap[&img.channel].name
        ));
        let res = reqwest::get(&img.url).await.context("downloading image")?;

        let bytes = res.bytes().await?;

        File::create(f)
            .context("creating output file")?
            .write_all(&bytes)
            .context("writing out image")?;

        count += 1;
    }

    Ok(count)
}

#[derive(Clone, serde::Serialize, Copy)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DLEvent {
    Started,
    Plane { r: u16, c: u16, f: u32, p: u16 },
    Finished,
}

#[tauri::command]
pub async fn start_download(
    on_event: Channel<DLEvent>,
    state: State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let state = state.lock().await;

    let hm = state.info.as_ref().ok_or("Missing XML Info")?;
    let filter = state.filter.as_ref().ok_or("Missing Filter")?;
    let outinfo = state.output.as_ref().ok_or("Missing output info")?;

    let imgs = filter.filter_images(&hm);

    on_event.send(DLEvent::Started).unwrap();

    max::max_project(&imgs, hm, &outinfo.dir, on_event.clone())
        .map_err(|e| format!("{:?}", e))
        .inspect(|_| on_event.send(DLEvent::Finished).unwrap())
}

#[tauri::command]
pub async fn test_download(outdir: &str, state: State<'_, Mutex<AppState>>) -> Result<u64, String> {
    let state = state.lock().await;
    let outdir = Path::new(outdir);

    match state.info {
        // AppState::ParsedXml(ref harmony) => download_image(&harmony, &outdir)
        //     .await
        //     .map_err(|e| format!("{:?}", e)),
        Some(ref hm) => dl_and_maxproj(hm, outdir)
            .context("max projecting images")
            .map_err(|e| format!("{:?}", e))
            .map(|_| hm.images.len() as u64),
        _ => Err("Bad app state".into()),
    }
}
