use anyhow::{Context, Result};
use ndarray::prelude::*;
use regex::Regex;
use serde::Deserialize;

use std::sync::LazyLock;

type RE = LazyLock<Regex>;

// needed to parse the FFC string as valid YAML..
// could just use str::replace, but whatever...
static RE_NULL: RE = LazyLock::new(|| Regex::new("Character: Null").unwrap());
const NULL_REPLACE: &str = r#"Character: "Null""#;

// static RE_COEFF: RE = LazyLock::new(|| Regex::new(r"Coefficients: \[\[(.*?)\]\]").unwrap());
// static RE_ORIGIN: RE = LazyLock::new(|| Regex::new(r"Origin: \[(.*?)\]").unwrap());
// static RE_SCALE: RE = LazyLock::new(|| Regex::new(r"Scale: \[(.*?)\]").unwrap());
// static RE_DIM: RE = LazyLock::new(|| Regex::new(r"Dims: \[(.*?)\]").unwrap());
// static RE_MEAN: RE = LazyLock::new(|| Regex::new(r"Mean: (.*?),").unwrap());

#[derive(Deserialize, Debug)]
#[serde(tag = "Type")]
enum Profile {
    Identity,
    #[serde(rename_all = "PascalCase")]
    Polynomial {
        coefficients: Vec<Vec<f64>>,
        dims: [f64; 2],
        origin: [f64; 2],
        scale: [f64; 2],
    },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(tag = "Character")]
enum Character {
    Null,
    Flat,
    #[serde(rename_all = "PascalCase")]
    NonFlat {
        mean: Option<f64>,
        noise_count: Option<f64>,
        non_flatness: NonFlatInfo,
    },
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "PascalCase")]
struct NonFlatInfo {
    corrected: Option<f64>,
    original: f64,
    random: f64,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "PascalCase")]
struct FieldInfo {
    #[serde(flatten)]
    character: Character,
    quality: f64,
    profile: Profile,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
#[serde(rename_all = "PascalCase")]
struct RawFFC {
    channel_name: String,
    channel: u8,
    version: String,
    background: FieldInfo,
    foreground: FieldInfo,
}

#[derive(Debug)]
pub enum FFC {
    // I think this is how it works,
    // with Basic only being background?
    // But a) double check with documentation
    // and b) what is all of the other info for?
    None,
    Basic(Array2<f64>),
    Advanced { bg: Array2<f64>, ff: Array2<f64> },
}

impl FFC {
    /// Parse out flat-field correction information
    /// if present for a wavelength in the export XML
    pub fn from_raw_xml(s: &str) -> Result<Self> {
        // avoid Character: Null issue
        let s = RE_NULL.replace_all(s, NULL_REPLACE);

        serde_yaml::from_str::<RawFFC>(&s)
            .map(|ffc| {
                let bg = convert_info_to_yx(&ffc.background);
                let ff = convert_info_to_yx(&ffc.foreground);

                match (bg, ff) {
                    (Some(bg), Some(ff)) => Self::Advanced { bg, ff },
                    (Some(bg), None) => Self::Basic(bg),
                    _ => Self::None,
                }
            })
            .with_context(|| format!("issue parsing YAML from:\n{}", s))
    }
}

fn convert_info_to_yx(info: &FieldInfo) -> Option<Array2<f64>> {
    match info.profile {
        Profile::Identity => None,
        Profile::Polynomial {
            ref coefficients,
            dims,
            origin,
            scale,
        } => {
            let [x, y] = generate_xy(dims, origin, scale);
            let c = convert_coeff(&coefficients);
            Some(polygrid2d(&x, &y, &c))
        }
    }
}

/// Create the x and y values to be applied to the cofficients
/// Based on: https://github.com/arronsullivan/Operetta_FFC
fn generate_xy(dims: [f64; 2], origin: [f64; 2], scale: [f64; 2]) -> [Array1<f64>; 2] {
    dims.into_iter()
        .zip(origin)
        .zip(scale)
        .map(|((d, o), s)| (Array1::range(0.0, d, 1.0) - o) * s)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn convert_coeff(arr: &[Vec<f64>]) -> Array2<f64> {
    let n = arr.len();
    let mut out = Array2::zeros([n, n]);

    for (i, row) in arr.iter().enumerate() {
        for (j, &coef) in row.iter().enumerate() {
            out[[i - j, j]] = coef;
        }
    }

    // convert from XY to YX
    out.reversed_axes()
}

/// Bad re-implementation of numpy's polygrid2
/// `x` and `y` are the paired values for x and y coefficients in `c`
/// https://github.com/numpy/numpy/blob/v2.2.0/numpy/polynomial/polynomial.py#L897-L950
/// https://github.com/numpy/numpy/blob/v2.2.0/numpy/polynomial/polyutils.py#L503-L516
/// https://github.com/numpy/numpy/blob/v2.2.0/numpy/polynomial/polynomial.py#L743-L755
fn polygrid2d(x: &Array1<f64>, y: &Array1<f64>, c: &Array2<f64>) -> Array2<f64> {
    let mut results = c.clone();
    let mut reshaped;

    for xi in [x, y] {
        reshaped = results.insert_axis(Axis(2));

        let mut c0 = x * 0.0 + reshaped.slice(s![-1, .., ..]);

        for i in 2..(reshaped.shape()[0] + 1) {
            let i = i as i32;
            c0 = c0 * xi + reshaped.slice(s![-i, .., ..]);
        }
        results = c0;
    }

    results
}
