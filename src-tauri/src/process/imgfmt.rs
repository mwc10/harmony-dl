use crate::parse_xml::{ChanMap, Harmony, Image};

#[derive(Copy, Clone)]
pub struct ImgNameFmt<'a> {
    // decimal digits for each
    r: usize,
    c: usize,
    t: usize,
    f: usize,
    p: usize,
    cmap: &'a ChanMap,
}

impl<'a> ImgNameFmt<'a> {
    pub fn fname_plane(&self, img: &Image) -> String {
        format!(
            "{}-R{:0rw$}C{:0cw$}T{:0tw$}F{:0fw$}P{:0pw$}",
            &self.cmap[&img.channel].name,
            img.row,
            img.col,
            img.timepoint,
            img.field,
            img.plane,
            rw = self.r,
            cw = self.c,
            tw = self.t,
            fw = self.f,
            pw = self.p
        )
    }
}

impl<'a> From<&'a Harmony> for ImgNameFmt<'a> {
    fn from(hm: &'a Harmony) -> Self {
        let numdig = |n: usize| (n.ilog10() + 1) as usize;
        Self {
            r: numdig(hm.plate.rows as usize),
            c: numdig(hm.plate.cols as usize),
            t: numdig(hm.timepoints as usize),
            f: numdig(hm.fields_per_well as usize),
            p: numdig(hm.planes_per_field as usize),
            cmap: &hm.channels,
        }
    }
}
