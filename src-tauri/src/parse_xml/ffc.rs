use ndarray::{Array2, Array};
use anyhow::{Result, Context};
use regex::Regex;

use std::sync::LazyLock;

type RE = LazyLock<Regex>;

static RE_COEFF: RE = LazyLock::new(|| Regex::new(r"Coefficients: \[\[(.*?)\]\]").unwrap());
static RE_ORIGIN: RE = LazyLock::new(|| Regex::new(r"Origin: \[(*.?)\]").unwrap());
static RE_SCALE: RE = LazyLock::new(|| Regex::new(r"Scale: \[(*.?)\]").unwrap());
static RE_DIM: RE = LazyLock::new(|| Regex::new(r"Dims: \[(*.?)\]").unwrap());
static RE_MEAN: RE = LazyLock::new(|| Regex::new(r"Mean: (*.?),").unwrap());

struct FFC {
    coeff: Array2<f64>,
    origin: (f64, f64),
    scale: (f64, f64),
    dim: (u32, u32),
    bg: f64,
}

impl FFC {
    fn from_xml_data(s: &str) -> Result<Self> {
        let mut coeff = Array::zeros((5, 5));
        
        todo!()
    }
}