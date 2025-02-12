use std::collections::HashSet;

use crate::parse_xml::{ChannelID, Harmony, Image};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ImageFilter {
    pub channels: HashSet<ChannelID>,
    // (row, col) [one indexed]
    pub wells: HashSet<(u16, u16)>,
    pub fields: HashSet<u32>,
    pub planes: HashSet<u16>,
}

impl ImageFilter {
    fn filter_harmony<'a>(&self, hm: &'a Harmony) -> Vec<&'a Image> {
        hm.images.iter()
            .filter(|img| {
                self.channels.contains(&img.channel) &
                self.wells.contains(&(img.row, img.col)) &
                self.fields.contains(&img.field) &
                self.planes.contains(&img.plane)
            }).collect()
    }
}

