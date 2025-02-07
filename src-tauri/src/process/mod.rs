use std::{fs::File, io::Write, path::Path};

use anyhow::{Context, Result};
use tauri::{async_runtime::Mutex, State};

use crate::{parse_xml::Harmony, AppState};

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
        AppState::ParsedXml(ref harmony) => download_image(&harmony, &outdir)
            .await
            .map_err(|e| format!("{:?}", e)),
        _ => Err("Bad app state".into()),
    }
}
