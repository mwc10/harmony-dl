use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Write},
    path::Path,
};

use anyhow::{Context, Result};
use image::{ImageBuffer, ImageFormat, ImageReader, Luma};
use ndarray::prelude::*;
use nshare::IntoNdarray2;
use rayon::prelude::*;
use tauri::{async_runtime::Mutex, State};

use crate::{
    parse_xml::{ChannelID, Harmony, Image},
    AppState,
};

type XY = Array2<u16>;
type Int16 = ImageBuffer<Luma<u16>, Vec<u16>>;

fn array_to_image(arr: XY) -> Int16 {
    let arr = arr.as_standard_layout().to_owned();
    let (h, w) = arr.dim();
    let (raw, _) = arr.into_raw_vec_and_offset();
    // image crate wants row order
    // https://stackoverflow.com/questions/56762026/how-to-save-ndarray-in-rust-as-image
    ImageBuffer::from_raw(w as u32, h as u32, raw).unwrap()
}

fn max_project(acc: Option<XY>, img: &Image) -> Result<Option<XY>> {
    let res = reqwest::blocking::get(&img.url).context("getting image")?;
    let raw = res.bytes().context("reading response bytes")?;

    let pixels = ImageReader::with_format(Cursor::new(&raw), ImageFormat::Tiff)
        .decode()
        .context("reading TIFF into pixels")?
        .into_luma16()
        .into_ndarray2();

    let acc = acc
        .map(|arr| {
            let stacked = ndarray::stack!(Axis(0), arr, pixels);
            stacked.map_axis(Axis(0), |view| *view.iter().max().unwrap())
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
            let ch = &cmap[&ch].name;
            let img = array_to_image(projection);
            let output = dir.join(format!("{ch}-R{r:02}C{c:02}F{f:03}T{tp}.tiff"));

            img.save_with_format(&output, ImageFormat::Tiff)
                .with_context(|| format!("saving image to {}", output.display()))
        })
}

async fn download_image(hm: &Harmony, dir: &Path) -> Result<u64> {
    let iter = hm
        .images
        .iter()
        .filter(|&img| img.row == 1 && img.col == 1 && img.timepoint == 1 && img.field == 1);

    let cmap = &hm.channels;
    let mut count = 0;
    for img in iter {
        let c = &cmap[&img.channel].name;
        let f = dir.join(format!(
            "R{:02}C{:02}F{:03}P{:03}-{c}.tiff",
            img.row, img.col, img.field, img.plane
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

#[tauri::command]
pub async fn test_download(outdir: &str, state: State<'_, Mutex<AppState>>) -> Result<u64, String> {
    let state = state.lock().await;
    let outdir = Path::new(outdir);

    match *state {
        // AppState::ParsedXml(ref harmony) => download_image(&harmony, &outdir)
        //     .await
        //     .map_err(|e| format!("{:?}", e)),
        AppState::ParsedXml(ref hm) => dl_and_maxproj(hm, outdir)
            .context("max projecting images")
            .map_err(|e| format!("{:?}", e))
            .map(|_| hm.images.len() as u64),
        _ => Err("Bad app state".into()),
    }
}
