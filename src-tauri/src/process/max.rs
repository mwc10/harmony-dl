use std::{collections::HashMap, fmt, io::Cursor, path::Path};

use anyhow::{anyhow, Context, Result};
use image::{ImageFormat, ImageReader};
use ndarray::azip;
use nshare::IntoNdarray2;
use rayon::iter::IntoParallelIterator;
use tauri::ipc::Channel;

use super::{array_to_image, DLEvent, YX};
use rayon::prelude::*;

use crate::parse_xml::{ChannelID, Harmony, Image};

/// Download an Image and project onto the accumulated pixels
fn max_field(acc: Option<YX>, img: &Image) -> Result<Option<YX>> {
    let res = reqwest::blocking::get(&img.url)
        .with_context(|| format!("downloading image from <{}>", &img.url))?;
    let raw = res
        .bytes()
        .context("reading response bytes from image download")?;

    // the image should be a 16bit intensity image, but maybe this can be configured dynamically?
    let pixels = ImageReader::with_format(Cursor::new(&raw), ImageFormat::Tiff)
        .decode()
        .context("reading raw bytes as TIFF image")?
        .into_luma16()
        .into_ndarray2();

    let acc = acc
        .map(|mut acc| {
            azip!((a in &mut acc, &b in &pixels) *a = (*a).max(b));
            acc
        })
        .or_else(|| Some(pixels));

    // report field finsihed

    Ok(acc)
}

#[derive(Hash, Copy, Clone, Eq, PartialEq)]
struct ImageKey {
    r: u16,
    c: u16,
    ch: ChannelID,
    t: u32,
    f: u32,
}

impl fmt::Display for ImageKey {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { r, c, ch, t, f } = self;
        write!(fmt, "Image <R{r}C{c}T{t}F{f} @ {ch:?}>")
    }
}

impl From<&Image> for ImageKey {
    fn from(img: &Image) -> Self {
        Self {
            r: img.row,
            c: img.col,
            ch: img.channel,
            t: img.timepoint,
            f: img.field,
        }
    }
}

/// Perform a maximum projection for each field. This downloads and projects the images
/// in parallel. For now, it outputs individual images in `outdir`, but this will evenutally
/// output either a directory images or one OME Zarr file.
pub fn max_project(
    imgs: &[&Image],
    hm: &Harmony,
    outdir: &Path,
    on_event: Channel<DLEvent>,
) -> Result<()> {
    type Map<'a> = HashMap<ImageKey, Vec<&'a Image>>;

    let cmap = &hm.channels;

    let by_field = imgs.into_iter().fold(Map::new(), |mut acc, &img| {
        let key = ImageKey::from(img);

        acc.entry(key)
            .and_modify(|arr| arr.push(img))
            .or_insert_with(|| vec![img]);

        acc
    });

    by_field
        .into_par_iter()
        .map(|(key, imgs)| {
            on_event
                .send(DLEvent::Started { r: key.r, c: key.c })
                .unwrap();

            imgs.into_iter()
                .try_fold(None, max_field)
                .map(|projection| (key, projection))
                .with_context(|| format!("processing {}", &key))
        })
        .try_for_each(|res| {
            let (key, projection) = res?;
            let projection = projection.ok_or_else(|| anyhow!("missing projection for {}", key))?;

            let ImageKey { r, c, ch, t, f } = key;
            let ch = cmap[&ch].name.as_str();
            let img = array_to_image(projection);
            // TODO: holder struct that has the max of each, then formats width dynamically
            let fname = format!("{ch}-R{r:02}C{c:02}T{t:03}F{f:03}.tiff");
            let output = outdir.join(&fname);

            let res = img
                .save_with_format(&output, ImageFormat::Tiff)
                .with_context(|| format!("saving projection to <{}>", output.display()));

            on_event
                .send(DLEvent::Finished { r: key.r, c: key.c })
                .unwrap();

            res
        })
}
