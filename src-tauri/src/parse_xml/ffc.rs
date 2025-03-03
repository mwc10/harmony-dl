use ndarray::{Array2, Array};
use anyhow::{anyhow, Context, Result};
use regex::Regex;

use std::sync::LazyLock;

type RE = LazyLock<Regex>;

static RE_COEFF: RE = LazyLock::new(|| Regex::new(r"Coefficients: \[\[(.*?)\]\]").unwrap());
static RE_ORIGIN: RE = LazyLock::new(|| Regex::new(r"Origin: \[(*.?)\]").unwrap());
static RE_SCALE: RE = LazyLock::new(|| Regex::new(r"Scale: \[(*.?)\]").unwrap());
static RE_DIM: RE = LazyLock::new(|| Regex::new(r"Dims: \[(*.?)\]").unwrap());
static RE_MEAN: RE = LazyLock::new(|| Regex::new(r"Mean: (*.?),").unwrap());

pub struct FFC {
    coeff: Array2<f64>,
    origin: (f64, f64),
    scale: (f64, f64),
    dim: (u32, u32),
    bg: f64,
}

impl FFC {
    pub fn from_xml_data(s: &str) -> Result<Self> {
        let get_first_group = |re: &RE| re.captures(s).and_then(|caps| caps.get(1)).map(|c| c.as_str());
        // TODO: dynamic based on parsed coefficients?
        let coeff = get_first_group(&RE_COEFF)
            .map(parse_coeffs)
            .transpose()
            .context("issue parsing coefficients")?
            .ok_or_else(|| anyhow!("missing coefficients string in XML data"))?;
 
        todo!()
    }
}

fn parse_coeffs(s: &str) -> Result<Array2<f64>> {
    
    let values = s.split("], [")
        .map(|sub| sub.split(", ").map(|v| v.parse::<f64>()).collect::<Result<Vec<_>, _>>())
        .collect::<Result<Vec<_>, _>>()
        .context("parsing coefficients into floats")?;
    
    let n = values.len();
    let mut output = Array2::zeros([n, n]);
    for (i, row) in values.into_iter().enumerate() {
        for (j , v) in row.into_iter().enumerate() {
            output[[i - j, j]] = v;
        }
    }

    Ok(output)
}
