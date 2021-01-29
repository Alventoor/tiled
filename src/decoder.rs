use std::fmt;
use std::str::FromStr;

use quick_xml::Reader;
use quick_xml::events::Event;
use quick_xml::events::attributes::Attributes;

use crate::data::*;

const MAP_TAG: &'static [u8] = b"map";
const TILESET_TAG: &'static [u8] = b"tileset";
const IMAGE_TAG: &'static [u8] = b"image";
const TILE_TAG: &'static [u8] = b"tile";
const GRID_TAG: &'static [u8]= b"grid";
const LAYER_TAG: &'static [u8] = b"layer";
const DATA_TAG: &'static [u8] = b"data";

/// Tente de convertir la chaîne de caractère dans le type de `buffer`. Si la
/// conversion réussie, enregistre la valeur dans `buffer`.
fn register_data<T>(buffer: &mut T, str: &str)
    where
        T: FromStr,
        <T as FromStr>::Err: fmt::Display,
{
    let data = str.parse::<T>();

    match data {
        Ok(d) => *buffer = d,
        Err(e) => eprintln!("Warning: {} (value: {})", e, str),
    }
}

/// Représente les différents états d'un fichier TMX lue.
#[derive(Clone, Debug)]
enum TMXState {
    /// On se situe dans le tag racine `map`.
    Map,
    /// On se situe dans le tag `tileset`.
    TileSet(TileSet),
    /// On se situe dans le tag `image` du parent `tileset`.
    TileSetImage(TileSet),
    /// On se situe dans le tag `tile` du parent `tileset`.
    Tile(TileSet, Tile),
    /// On se situe dans le tag `image` du parent `tile`.
    TileImage(TileSet, Tile),
    /// On se situe dans le tag `grid` du parent `tileset`.
    Grid(TileSet),
    /// On se situe dans le tag `layer`.
    Layer,
    /// On se situe dans le tag `data` du parent `layer`.
    Data,
    /// Tag inconnue. Sa donnée représente son parent.
    Unknown(Box<TMXState>),
}

impl TMXState {
    /// Consomme l'énumération, renvoyant l'enfant associé portant le nom passé
    /// en paramètre.
    ///
    /// Si aucun enfant ne correspond, renvoie `Unknown`.
    pub fn into_child(self, name: &[u8]) -> Self {
        match self {
            Self::Map if name == MAP_TAG => Self::Map,
            Self::Map if name == TILESET_TAG => Self::TileSet(TileSet::default()),
            Self::Map if name == LAYER_TAG => Self::Layer,
            Self::TileSet(tileset) if name == IMAGE_TAG => Self::TileSetImage(tileset),
            Self::TileSet(tileset) if name == TILE_TAG => Self::Tile(tileset, Tile::default()),
            Self::TileSet(tileset) if name == GRID_TAG => Self::Grid(tileset),
            Self::Tile(tileset, tile) if name == IMAGE_TAG => Self::TileImage(tileset, tile),
            Self::Layer if name == DATA_TAG => Self::Data,
            _ => Self::Unknown(Box::new(self)),
        }
    }

    /// Consomme l'énumération, renvoyant le parent associé.
    pub fn into_parent(self) -> Self {
        match self {
            Self::Map => self,
            Self::TileSet(_) | Self::Layer => Self::Map,
            Self::Tile(tileset, _)
            | Self::TileSetImage(tileset)
            | Self::Grid(tileset) => Self::TileSet(tileset),
            Self::TileImage(tileset, tile) => Self::Tile(tileset, tile),
            Self::Data =>Self::Layer,
            Self::Unknown(state) => *state,
        }
    }
}

pub struct TMXDecoder<'a> {
    xml_reader: Reader<&'a [u8]>,
    state: TMXState,
}

impl<'a> TMXDecoder<'a> {
    #[inline]
    pub fn from<B: AsRef<[u8]>>(data: &'a B) -> Self {
        Self::_from(data.as_ref())
    }

    fn _from(data: &'a [u8]) -> Self {
        let mut xml_reader = Reader::from_reader(data);
        xml_reader.trim_text(true);

        Self {
            xml_reader,
            state: TMXState::Map,
        }
    }

    fn tag(&mut self, attributes: &mut Attributes, map: &mut Map) {
        match self.state {
            TMXState::Map => map_tag(attributes, map),
            TMXState::TileSet(ref mut tileset) => tileset_tag(tileset, attributes),
            TMXState::TileSetImage(ref mut tileset) => tileset_image_tag(tileset, attributes),
            TMXState::Tile(_, ref mut tile) => tile_tag(tile, attributes),
            TMXState::TileImage(_, ref mut tile) => tile_image_tag(tile, attributes),
            _ => { /* Nothing to do */ }
        }
    }

    fn text(&mut self, bytes: &[u8], map: &mut Map) {
        if let TMXState::Data = self.state {
            data_text(bytes, map);
        }
    }

    fn end(&mut self, map: &mut Map) {
        let state = std::mem::replace(&mut self.state, TMXState::Map);

        self.state = match state {
            TMXState::TileSet(tileset) => {
                map.add_tileset_without_reordering(tileset);
                TMXState::Map
            }
            TMXState::Tile(mut tileset, tile) => {
                tileset.origin.insert_collection(tile);
                TMXState::TileSet(tileset)
            }
            _ => state.into_parent()
        };
    }

    pub fn load_map(mut self) -> Map {
        let mut map = Map::default();
        let mut buffer = Vec::new();

        loop {
            match self.xml_reader.read_event(&mut buffer) {
                Ok(Event::Start(ref b)) => {
                    self.state = self.state.into_child(b.name());
                    self.tag(&mut b.attributes(), &mut map);
                }
                Ok(Event::Empty(ref b)) => {
                    self.state = self.state.into_child(b.name());
                    self.tag(&mut b.attributes(), &mut map);
                    self.end(&mut map);
                }
                Ok(Event::Text(ref b)) => self.text(b.escaped(), &mut map),
                Ok(Event::End(_)) => self.end(&mut map),
                Ok(Event::Eof) => break,
                Err(e) => panic!("Error at position {} : {:?}", self.xml_reader.buffer_position(), e),
                _ => {}
            }

            buffer.clear();
        }

        map.reorder_tilesets();
        map
    }
}

fn extract_image_path(attributes: &mut Attributes) -> Option<String> {
    attributes
        .filter_map(|a| a.ok())
        .find(|a| a.key == b"source")
        .map(|a| String::from_utf8_lossy(&a.value).trim_start_matches("../").to_string())
}

fn map_tag(attributes: &mut Attributes, map: &mut Map) {
    for attribute in attributes.filter_map(|a| a.ok()) {
        if let Ok(value) = std::str::from_utf8(&attribute.value) {
            match attribute.key {
                b"width" => register_data(&mut map.size.width, value),
                b"height" => register_data(&mut map.size.height, value),
                b"tilewidth" => register_data(&mut map.tile_size.width, value),
                b"tileheight" => register_data(&mut map.tile_size.height, value),
                b"orientation" => register_data(&mut map.orientation, value),
                b"staggeraxis" => register_data(&mut map.stagger_axis, value),
                _ => { /* Nothing to do */ }
            }
        }
    }
}

fn tileset_tag(tileset: &mut TileSet, attributes: &mut Attributes) {
    for attribute in attributes.filter_map(|a| a.ok()) {
        if let Ok(value) = std::str::from_utf8(&attribute.value) {
            match attribute.key {
                b"firstgid" => register_data(&mut tileset.firstgid, value),
                b"tilewidth" => register_data(&mut tileset.size.width, value),
                b"tileheight" => register_data(&mut tileset.size.height, value),
                b"tilecount" => register_data(&mut tileset.count, value),
                b"columns" => register_data(&mut tileset.columns, value),
                b"name" => register_data(&mut tileset.name, value),
                _ => { /* Nothing to do */ }
            }
        }
    }
}

fn tileset_image_tag(tileset: &mut TileSet, attributes: &mut Attributes) {
    if let Some(path) = extract_image_path(attributes) {
        tileset.origin = TileOrigin::Image(path);
    }
}

fn tile_tag(tile: &mut Tile, attributes: &mut Attributes) {
    let encoded_data = attributes
        .filter_map(|a| a.ok())
        .find(|a| a.key == b"id")
        .map(|a| String::from_utf8_lossy(&a.value).into_owned());

    if let Some(d) = encoded_data {
        register_data(&mut tile.id, &d);
    }
}

fn tile_image_tag(tile: &mut Tile, attributes: &mut Attributes) {
    if let Some(path) = extract_image_path(attributes) {
        tile.image_path = path;
    }
}

fn data_text(bytes: &[u8], map: &mut Map) {
    let encoded_data = String::from_utf8_lossy(&bytes);

    map.tiles = encoded_data
        .split(|c: char| c == ',' || c == '\n')
        .filter_map(|d| d.parse::<u32>().ok())
        .collect();
}
