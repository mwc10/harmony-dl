mod filter;
mod imgfmt;
mod individual;
mod max;

pub use filter::ImageFilter;

use anyhow::{anyhow, Result};
use image::{ImageBuffer, Luma};
use ndarray::prelude::*;
use tauri::{async_runtime::Mutex, ipc::Channel, State};

use crate::{parse_xml::Image, AppState};

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct OutputInfo {
    pub dir: std::path::PathBuf,
    pub action: OutputAction,
    pub format: OutputFormat,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum OutputAction {
    #[serde(rename = "Max Projection")]
    MaxProjection,
    #[serde(rename = "Individual Planes")]
    IndividualPlanes,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy)]
pub enum OutputFormat {
    #[serde(rename = "TIFF")]
    Tiff,
    #[serde(rename = "OME-Zarr")]
    OmeZarr,
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

#[derive(Clone, serde::Serialize, Copy)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum DLEvent {
    Started,
    Plane { r: u16, c: u16, f: u32, p: u16 },
    Finished,
}

impl From<&Image> for DLEvent {
    fn from(img: &Image) -> Self {
        Self::Plane {
            r: img.row,
            c: img.col,
            f: img.field,
            p: img.plane,
        }
    }
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

    on_event
        .send(DLEvent::Started)
        .map_err(|_| "sending start DL event")?;

    let res = match outinfo.action {
        OutputAction::MaxProjection => max::max_project(&imgs, hm, &outinfo.dir, on_event.clone()),
        OutputAction::IndividualPlanes => {
            individual::download_tiff_images(&imgs, hm, &outinfo.dir, on_event.clone())
        }
    };

    res.map_err(|e| format!("{:?}", e))
        .inspect(|_| on_event.send(DLEvent::Finished).unwrap())
}
