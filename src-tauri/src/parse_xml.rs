use super::AppState;
use anyhow::{anyhow, bail, Context, Result};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{BufReader, Read},
    path::Path,
};
use tauri::{async_runtime::Mutex, State};

// Helpers
type TempMap = HashMap<String, String>;
pub type ChanMap = HashMap<ChannelID, Channel>;
type PlateVec<T> = Vec<Vec<Option<T>>>;
type PlateMap<T> = HashMap<(u8, u8), T>;

fn get_from<'a>(map: &'a TempMap, key: &str) -> Result<&'a String> {
    map.get(key).ok_or_else(|| anyhow!("Missing key <{}>", key))
}

fn get_string<'a>(map: &'a TempMap, key: &str) -> Result<String> {
    get_from(map, key).map(String::to_string)
}

fn get_u8<'a>(map: &'a TempMap, key: &str) -> Result<u8> {
    get_from(map, key).and_then(|s| s.parse::<u8>().context("parsing as u8"))
}

fn get_u16<'a>(map: &'a TempMap, key: &str) -> Result<u16> {
    get_from(map, key).and_then(|s| s.parse::<u16>().context("parsing as u16"))
}

fn get_u32<'a>(map: &'a TempMap, key: &str) -> Result<u32> {
    get_from(map, key).and_then(|s| s.parse::<u32>().context("parsing as u32"))
}

fn get_f64<'a>(map: &'a TempMap, key: &str) -> Result<f64> {
    get_from(map, key).and_then(|s| s.parse::<f64>().context("parsing as f64"))
}

/// Harmony defined channel IDs...
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, serde::Serialize)]
pub struct ChannelID(u8);

/// Holds all of the necessary information from
/// the harmony export XML file
/// I can't figure out how multiple plates work, so
/// I've reduced it to only having one plate...
#[derive(Debug)]
pub struct Harmony {
    pub plate: Plate,
    pub channels: ChanMap,
    pub images: Vec<Image>,
    pub wells: PlateMap<WellInfo>,
}

impl Harmony {
    // TODO: async read from tokio...? as stand alone function probably...
    fn from_xml_path(p: &Path) -> Result<Self> {
        use xml::reader::XmlEvent::{EndDocument, StartElement};

        let f = File::open(p).context("opening XML file")?;
        let mut rdr = xml::ParserConfig::new()
            .trim_whitespace(true)
            .ignore_comments(true)
            .cdata_to_characters(true)
            .create_reader(BufReader::new(f));

        let mut plate = None;
        let mut channels = None;
        let mut images = None;
        let mut wells = None;

        loop {
            let evt = rdr.next().context("getting next XML event")?;

            match evt {
                StartElement { name, .. } if name.local_name == "Plates" => {
                    plate = parse_plates(&mut rdr)
                        .map(Some)
                        .context("parsing <Plates>")?;
                }
                StartElement { name, .. } if name.local_name == "Maps" => {
                    channels = parse_maps(&mut rdr)
                        .map(Some)
                        .context("parsing channel info from <Maps>")?;
                }
                StartElement { name, .. } if name.local_name == "Images" => {
                    images = parse_images(&mut rdr)
                        .map(Some)
                        .context("parsing <Images>")?;
                    wells = images.as_deref().map(summarize_images);
                }
                EndDocument => break,
                _ => (),
            }
        }

        // there has to be better way to do this..? map_n? match?
        plate
            .zip(channels)
            .zip(images)
            .zip(wells)
            .map(|(((plate, channels), images), wells)| Self {
                plate,
                channels,
                images,
                wells,
            })
            .ok_or_else(|| anyhow!("Missing components in XML file"))
    }
}

/// Imaging plate information
#[derive(Debug)]
pub struct Plate {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub rows: u16,
    pub cols: u16,
}

impl TryFrom<TempMap> for Plate {
    type Error = anyhow::Error;

    fn try_from(value: TempMap) -> Result<Self> {
        let get_str = |key| get_string(&value, key).context("parsing Plate");
        let get_u16 = |key| get_u16(&value, key).context("parsing Plate");

        Ok(Self {
            id: get_str("PlateID")?,
            name: get_str("Name")?,
            kind: get_str("PlateTypeName")?,
            rows: get_u16("PlateRows")?,
            cols: get_u16("PlateColumns")?,
        })
    }
}

// TODO: FlatField correction info in <FlatfieldProfile>
#[derive(Debug, serde::Serialize, Clone)]
pub struct Channel {
    pub id: ChannelID,
    pub name: String,
    pub res: (f64, f64), // in microns
    pub mag: u16,
    //pub flatfield_profile: Vec<u8>,
}

impl TryFrom<(TempMap, ChannelID)> for Channel {
    type Error = anyhow::Error;

    fn try_from(value: (TempMap, ChannelID)) -> std::result::Result<Self, Self::Error> {
        let (value, id) = value;

        let get_str =
            |key| get_string(&value, key).with_context(|| format!("parsing Channel {}", id.0));
        let get_u16 =
            |key| get_u16(&value, key).with_context(|| format!("parsing Channel {}", id.0));
        let get_f64 =
            |key| get_f64(&value, key).with_context(|| format!("parsing Channel {}", id.0));

        Ok(Self {
            id: id,
            name: get_str("ChannelName")?,
            // originally, these are in meters? do this check dynamically?
            res: (
                get_f64("ImageResolutionX")? * 1e6,
                get_f64("ImageResolutionY")? * 1e6,
            ),
            mag: get_u16("ObjectiveMagnification")?,
        })
    }
}

#[derive(Debug)]
pub struct Image {
    pub row: u16,
    pub col: u16,
    pub field: u32,
    pub plane: u16,
    pub timepoint: u32,
    pub channel: ChannelID,
    pub url: String,
    pub position: [f64; 4], // [x, y, z, abs_z] all in meters... until dynamic?
}

impl TryFrom<TempMap> for Image {
    type Error = anyhow::Error;

    fn try_from(value: TempMap) -> std::result::Result<Self, Self::Error> {
        let get_str = |key| get_string(&value, key).context("parsing Image");
        let get_u8 = |key| get_u8(&value, key).context("parsing Image");
        let get_u16 = |key| get_u16(&value, key).context("parsing Image");
        let get_u32 = |key| get_u32(&value, key).context("parsing Image");
        let get_f64 = |key| get_f64(&value, key).context("parsing Image");

        let position = [
            get_f64("PositionX")?,
            get_f64("PositionY")?,
            get_f64("PositionZ")?,
            get_f64("AbsPositionZ")?,
        ];

        Ok(Self {
            row: get_u16("Row")?,
            col: get_u16("Col")?,
            field: get_u32("FieldID")?,
            plane: get_u16("PlaneID")?,
            timepoint: get_u32("TimepointID")?,
            channel: get_u8("ChannelID").map(ChannelID)?,
            url: get_str("URL")?,
            position,
        })
    }
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct WellInfo {
    pub row: u8,
    pub col: u8,
    pub fields: HashSet<u32>,
    pub planes: HashSet<u16>,
    pub timepoints: HashSet<u32>,
}

impl WellInfo {
    fn new(row: u8, col: u8) -> Self {
        Self {
            row,
            col,
            ..Default::default()
        }
    }
}

// ###### Parser Functions ######
fn parse_plates<R: Read>(rdr: &mut xml::EventReader<R>) -> Result<Plate> {
    use xml::reader::XmlEvent::*;

    let mut output = vec![];
    let mut state: Option<TempMap> = None;
    let mut field: Option<String> = None;

    loop {
        match rdr.next()? {
            StartElement { name, .. } if name.local_name == "Plate" => {
                if state.is_some() {
                    bail!("Plate tag opened without finishing prior plate");
                } else {
                    state = Some(HashMap::new());
                }
            }
            // skip the well tag
            StartElement { name, .. } if name.local_name == "Well" => (),

            // fields of the struct, basically...
            StartElement { name, .. } => field = Some(name.local_name),
            Characters(data) => {
                let k = field
                    .take()
                    .ok_or_else(|| anyhow!("Missing field for data: {}", &data))?;

                state = state.map(|mut s| {
                    s.insert(k, data);
                    s
                });
                // error for no state?
            }
            // don't need EndElement since we took the name of the field

            // plate end
            EndElement { name } if name.local_name == "Plate" => {
                let plate = state
                    .take()
                    .ok_or_else(|| anyhow!("Missing state when plate tag closed"))
                    .and_then(Plate::try_from)
                    .context("issusing converting dict into Plate")?;

                output.push(plate);
            }
            // array of plates end
            EndElement { name } if name.local_name == "Plates" => {
                break;
            }

            // ignore everything else...
            _ => (),
        }
    }

    if output.len() > 1 {
        bail!(
            "Found more than 1 plate in <Plates> section: {}",
            output.len()
        );
    }

    output
        .pop()
        .ok_or_else(|| anyhow!("Found no plates in <Plates> section"))
}

fn parse_maps<R: Read>(rdr: &mut xml::EventReader<R>) -> Result<ChanMap> {
    use xml::reader::XmlEvent::*;

    fn find_channel_id(attr: &[xml::attribute::OwnedAttribute]) -> Result<ChannelID> {
        attr.iter()
            .find(|a| a.name.local_name == "ChannelID")
            .ok_or_else(|| anyhow!("No ChannelID in Entry attributes"))
            .and_then(|a| a.value.parse::<u8>().context("parsing channel id"))
            .map(ChannelID)
    }

    let mut raw: HashMap<ChannelID, TempMap> = HashMap::new();
    let mut channel: Option<ChannelID> = None;
    let mut field: Option<String> = None;

    loop {
        match rdr.next()? {
            // get the channel id
            // <Entry ChannelID=''>
            StartElement {
                name, attributes, ..
            } if name.local_name == "Entry" => {
                channel = Some(find_channel_id(&attributes)?);
            }
            // get the field name if in an entry
            StartElement { name, .. } if channel.is_some() => field = Some(name.local_name),
            // put info into the map if there is a channel and a field
            Characters(val) if channel.is_some() && field.is_some() => {
                let chan = channel.clone().unwrap();
                let key = field.take().unwrap();
                let map = raw.entry(chan).or_default();
                map.insert(key, val);
            }

            // </Entry>
            EndElement { name } if name.local_name == "Entry" => {
                channel = None;
            }
            // </Maps>
            EndElement { name } if name.local_name == "Maps" => {
                break;
            }
            _ => (),
        }
    }

    raw.into_iter()
        .map(|(k, temp)| Channel::try_from((temp, k)).map(|v| (k, v)))
        .collect()
}

fn parse_images<R: Read>(rdr: &mut xml::EventReader<R>) -> Result<Vec<Image>> {
    use xml::reader::XmlEvent::*;

    let mut output = Vec::with_capacity(1024);
    let mut state: Option<TempMap> = None;
    let mut field: Option<String> = None;

    loop {
        match rdr.next()? {
            // <Image Version="">
            StartElement { name, .. } if name.local_name == "Image" => {
                state = Some(TempMap::with_capacity(32));
            }
            // <Field>
            StartElement { name, .. } if state.is_some() => field = Some(name.local_name),

            Characters(v) if field.is_some() && state.is_some() => {
                let k = field.take().unwrap();
                state = state.map(|mut s| {
                    s.insert(k, v);
                    s
                });
            }

            // </Image>
            EndElement { name } if name.local_name == "Image" => {
                let s = state
                    .take()
                    .ok_or_else(|| anyhow!("Missing state after closing Image tag"))?;

                output.push(Image::try_from(s).context("parsing data to Image")?);
            }

            // </Images>
            EndElement { name } if name.local_name == "Images" => {
                break;
            }
            _ => (),
        }
    }

    Ok(output)
}

// ### Summarization Functions ###
fn summarize_images(imgs: &[Image]) -> PlateMap<WellInfo> {
    let wells: HashMap<(u8, u8), WellInfo> = HashMap::new();

    imgs.into_iter()
        .map(|img| (img.row as u8, img.col as u8, img))
        .fold(wells, |mut wells, (r, c, img)| {
            let w = wells.entry((r, c)).or_insert_with(|| WellInfo::new(r, c));

            w.fields.insert(img.field);
            w.planes.insert(img.plane);
            w.timepoints.insert(img.timepoint);

            wells
        })
}

// ###### Tauri Glue ######
#[derive(Debug, serde::Serialize)]
pub struct XmlInfo {
    name: String,
    rows: u8,
    cols: u8,
    fields: u16,
    planes: u16,
    timepoints: u16,
    wells: PlateVec<WellInfo>,
    channels: Vec<Channel>,
    // problem_wells? something missing fields or stacks...
}

impl From<&Harmony> for XmlInfo {
    fn from(h: &Harmony) -> Self {
        let (r, c) = (h.plate.rows as usize, h.plate.cols as usize);
        let (mut fields, mut planes, mut timepoints) = (0, 0, 0);
        let mut wells = vec![vec![None; c]; r];

        for (&(r, c), well) in h.wells.iter() {
            let (r, c) = (r as usize, c as usize);
            let (r, c) = (r - 1, c - 1);
            fields = fields.max(well.fields.len());
            planes = planes.max(well.planes.len());
            timepoints = timepoints.max(well.timepoints.len());

            wells[r][c] = Some(well.clone());
        }

        let mut channels: Vec<Channel> = h.channels.values().cloned().collect();
        channels.sort_by(|a, b| a.id.cmp(&b.id));

        Self {
            name: h.plate.name.clone(),
            rows: r as u8,
            cols: c as u8,
            fields: fields as u16,
            planes: planes as u16,
            timepoints: timepoints as u16,
            wells,
            channels,
        }
    }
}

#[tauri::command]
pub async fn parse_xml(path: &str, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let path = Path::new(path);

    let info = Harmony::from_xml_path(&path).map_err(|e| format!("{:?}", e))?;

    // store state so that images from selected wells can be fetched later
    let mut state = state.lock().await;
    *state = AppState::ParsedXml(info);

    Ok(())
}

#[tauri::command]
pub async fn get_info(state: State<'_, Mutex<AppState>>) -> Result<XmlInfo, String> {
    let state = state.lock().await;

    match *state {
        AppState::Started => Err("App has not yet generated Harmony Information".into()),
        AppState::ParsedXml(ref h) => Ok(XmlInfo::from(h)),
    }
}
