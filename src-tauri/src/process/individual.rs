use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use rayon::prelude::*;
use tauri::ipc::Channel;

use crate::parse_xml::{Harmony, Image};

use super::{imgfmt::ImgNameFmt, DLEvent};

type Info<'a> = (Channel<DLEvent>, &'a Path);
type Img<'a> = (&'a Image, String);

fn dl_tiff((evt, outdir): &mut Info, (img, fname): Img) -> Result<()> {
    let raw = reqwest::blocking::get(&img.url)
        .and_then(|res| res.bytes())
        .with_context(|| format!("dowloading image <{}> ({})", &img.url, &fname))?;

    // TODO: flat field correction....
    // open image -> NDarray YX -> FFC -> Save as TIFF
    let mut fname = PathBuf::from(fname);
    fname.set_extension("tiff");
    let output = outdir.join(&fname);

    fs::File::create(output)
        .with_context(|| format!("creating output <{}>", fname.display()))
        .and_then(|mut f| {
            f.write_all(&raw)
                .with_context(|| format!("writing raw bytes output <{}>", fname.display()))
        })
        .and_then(|_| {
            evt.send(DLEvent::from(img))
                .context("sending download progress")
        })
}

pub fn download_tiff_images(
    imgs: &[&Image],
    hm: &Harmony,
    outdir: &Path,
    event: Channel<DLEvent>,
) -> Result<()> {
    let fmt = ImgNameFmt::from(hm);
    imgs.into_par_iter()
        .map(|&img| (img, fmt.fname_plane(img)))
        .try_for_each_with((event, outdir), dl_tiff)
        .context("dowloading image")
}
