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
    /// On se situe dans le tag racine `<map>`.
    Map,
    /// On se situe dans le tag `<tileset>`.
    TileSet(TileSet),
    /// On se situe dans le tag `<image>` du parent `<tileset>`.
    TileSetImage(TileSet),
    /// On se situe dans le tag `<tile>` du parent `<tileset>`.
    Tile(TileSet, Tile),
    /// On se situe dans le tag `<image>` du parent `<tile>`.
    TileImage(TileSet, Tile),
    /// On se situe dans le tag `<grid>` du parent `<tileset>`.
    Grid(TileSet),
    /// On se situe dans le tag `<layer>`.
    Layer,
    /// On se situe dans le tag `<data>` du parent `<layer>`.
    Data,
    /// Tag inconnu. Sa donnée représente son parent.
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

/// Permet la désérialisation du contenu d'un fichier .tmx au sein d'une
/// structure [`Map`].
///
/// [`Map`]: ./struct.Map.html
pub struct TMXDecoder<'a> {
    xml_reader: Reader<&'a [u8]>,
    state: TMXState,
}

impl<'a> TMXDecoder<'a> {
    /// Crée un nouveau décodeur travaillant sur les données passées en paramètre.
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

    /// Récupère au sein de la map les données associées aux attributs du tag
    /// actuellement lu.
    fn tag(&mut self, attributes: &mut Attributes, map: &mut Map) {
        match self.state {
            TMXState::Map => map_tag(attributes, map),
            TMXState::TileSet(ref mut tileset) => tileset_tag(attributes, tileset),
            TMXState::TileSetImage(ref mut tileset) => tileset_image_tag(attributes, tileset),
            TMXState::Tile(_, ref mut tile) => tile_tag(attributes, tile),
            TMXState::TileImage(_, ref mut tile) => tile_image_tag(attributes, tile),
            _ => { /* Nothing to do */ }
        }
    }

    /// Récupère au sein de la map les données associées au texte du tag
    /// actuellement lu.
    fn text(&mut self, bytes: &[u8], map: &mut Map) {
        if let TMXState::Data = self.state {
            data_text(bytes, map);
        }
    }

    /// Quitte le tag actuel, en exécutant diverses action de fin selon celui-ci.
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

    /// Charge une nouvelle map à l'aide des données du décodeur.
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

/// Récupère le chemin d'accès d'une image si présent parmi les attributs.
///
/// La clef associée au chemin d'accès doit être "source".
/// Exemple : <.. source="path">
fn extract_image_path(attributes: &mut Attributes) -> Option<String> {
    attributes
        .filter_map(|a| a.ok())
        .find(|a| a.key == b"source")
        .map(|a| String::from_utf8_lossy(&a.value).into_owned())
}

/// Récupère les paramètres de la map associés au tag `<map>` si présents dans
/// la liste d'attributs.
fn map_tag(attributes: &mut Attributes, map: &mut Map) {
    for attribute in attributes.filter_map(|a| a.ok()) {
        if let Ok(value) = std::str::from_utf8(&attribute.value) {
            match attribute.key {
                b"width" => register_data(&mut map.size.x, value),
                b"height" => register_data(&mut map.size.y, value),
                b"tilewidth" => register_data(&mut map.tile_size.x, value),
                b"tileheight" => register_data(&mut map.tile_size.y, value),
                b"orientation" => register_data(&mut map.orientation, value),
                b"staggeraxis" => register_data(&mut map.stagger_axis, value),
                _ => { /* Nothing to do */ }
            }
        }
    }
}

/// Récupère les paramètres de jeu de tuiles associés au tag `<tileset>` si
/// présents dans la liste d'attributs.
fn tileset_tag(attributes: &mut Attributes, tileset: &mut TileSet) {
    for attribute in attributes.filter_map(|a| a.ok()) {
        if let Ok(value) = std::str::from_utf8(&attribute.value) {
            match attribute.key {
                b"firstgid" => register_data(&mut tileset.firstgid, value),
                b"tilewidth" => register_data(&mut tileset.size.x, value),
                b"tileheight" => register_data(&mut tileset.size.y, value),
                b"tilecount" => register_data(&mut tileset.count, value),
                b"columns" => register_data(&mut tileset.columns, value),
                b"name" => register_data(&mut tileset.name, value),
                _ => { /* Nothing to do */ }
            }
        }
    }
}

/// Récupère le chemin d'accès relatif de l'image d'un jeu de tuiles associé au
/// tag `<tileset><image ../></tileset>` si présent dans la liste d'attributs.
fn tileset_image_tag(attributes: &mut Attributes, tileset: &mut TileSet) {
    if let Some(path) = extract_image_path(attributes) {
        tileset.origin = TilesOrigin::Image(path);
    }
}

/// Récupère les paramètres de tuile associés au tag `<tile>` si présents dans
/// la liste d'attributs.
fn tile_tag(attributes: &mut Attributes, tile: &mut Tile) {
    let encoded_data = attributes
        .filter_map(|a| a.ok())
        .find(|a| a.key == b"id")
        .map(|a| String::from_utf8_lossy(&a.value).into_owned());

    if let Some(d) = encoded_data {
        register_data(&mut tile.id, &d);
    }
}

/// Récupère le chemin d'accès de l'image d'une tuile associé au tag
/// `<tile><image ../></tile>` si présent dans la liste d'attributs.
fn tile_image_tag(attributes: &mut Attributes, tile: &mut Tile) {
    if let Some(path) = extract_image_path(attributes) {
        tile.image_path = path;
    }
}

/// Récupère les identifiants globaux des tuiles présents au sein du tag
/// `<data>`.
fn data_text(bytes: &[u8], map: &mut Map) {
    let encoded_data = String::from_utf8_lossy(&bytes);

    map.tiles = encoded_data
        .split(|c: char| c == ',' || c == '\n')
        .filter_map(|d| d.parse().ok())
        .collect();
}

#[cfg(test)]
mod tests {
    use quick_xml::events::attributes::Attributes;
    use mint::Vector2;
    use super::*;

    #[test]
    fn register_data_test() {
        let correct_nb = 16;
        let str = correct_nb.to_string();

        let mut nb = 0;

        register_data(&mut nb, &str);
        assert_eq!(nb, correct_nb);
    }

    #[test]
    fn extract_image_path_test() {
        let buf = b"< source=\"path\" >";
        let mut attributes = Attributes::new(buf, 0);

        let path = extract_image_path(&mut attributes);
        assert_eq!(path, Some(String::from("path")));
    }

    #[test]
    fn map_tag_test() {
        let mut correct_map = Map::default();
        correct_map.size = Vector2 { x: 10, y: 8 };
        correct_map.tile_size = Vector2 { x: 32, y: 16 };
        correct_map.orientation = Orientation::Orthogonal;
        correct_map.stagger_axis = StaggerAxis::XAxis;

        let buf = format!(
            "< width=\"{}\" height=\"{}\", tilewidth=\"{}\" tileheight=\"{}\" orientation=\"orthogonal\", staggeraxis=\"x\" >",
            correct_map.size.x, correct_map.size.y,
            correct_map.tile_size.x, correct_map.tile_size.y
        );

        let mut map = Map::default();
        let mut attributes = Attributes::new(buf.as_bytes(), 0);

        map_tag(&mut attributes, &mut map);
        assert_eq!(map.size, correct_map.size);
        assert_eq!(map.tile_size, correct_map.tile_size);
        assert_eq!(map.orientation, correct_map.orientation);
        assert_eq!(map.stagger_axis, correct_map.stagger_axis);
    }

    #[test]
    fn tileset_tag_test() {
        let correct_tileset = TileSet {
            firstgid: 1,
            size: Vector2 { x: 64, y: 32 },
            count: 6,
            columns: 2,
            name: String::from("correct tileset"),
            origin: TilesOrigin::None
        };

        let buf = format!(
            "< firstgid=\"{}\" tilewidth=\"{}\" tileheight=\"{}\" tilecount=\"{}\", columns=\"{}\" name=\"{}\" >",
            correct_tileset.firstgid, correct_tileset.size.x,
            correct_tileset.size.y, correct_tileset.count,
            correct_tileset.columns, correct_tileset.name
        );

        let mut tileset = TileSet::default();
        let mut attributes = Attributes::new(buf.as_bytes(), 0);

        tileset_tag(&mut attributes, &mut tileset);
        assert_eq!(tileset.firstgid, correct_tileset.firstgid);
        assert_eq!(tileset.size, correct_tileset.size);
        assert_eq!(tileset.count, correct_tileset.count);
        assert_eq!(tileset.columns, correct_tileset.columns);
        assert_eq!(tileset.name, correct_tileset.name);
    }

    #[test]
    fn tileset_image_tag_test() {
        let correct_origin = TilesOrigin::Image(String::from("path"));
        let buf = b"< source=\"path\" >";

        let mut tileset = TileSet::default();
        let mut attributes = Attributes::new(buf, 0);

        tileset_image_tag(&mut attributes, &mut tileset);
        assert_eq!(tileset.origin, correct_origin);
    }

    #[test]
    fn tile_tag_test() {
        let correct_id = 2;
        let buf = b"< id=\"2\" >";

        let mut tile = Tile::default();
        let mut attributes = Attributes::new(buf, 0);

        tile_tag(&mut attributes, &mut tile);
        assert_eq!(tile.id, correct_id);
    }

    #[test]
    fn tile_image_tag_test() {
        let correct_path = String::from("path");
        let buf = b"< source=\"path\" >";

        let mut tile = Tile::default();
        let mut attributes = Attributes::new(buf, 0);

        tile_image_tag(&mut attributes, &mut tile);
        assert_eq!(tile.image_path, correct_path);
    }

    #[test]
    fn data_text_test() {
        let mut correct_map = Map::default();
        correct_map.tiles = vec![0, 0, 0, 3, 2, 1];

        let mut map = Map::default();
        let bytes = b"0,0,0,\n3,2,1";

        data_text(bytes, &mut map);
        assert_eq!(map.tiles, correct_map.tiles);
    }
}