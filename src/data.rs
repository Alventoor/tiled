use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;

use mint::{Point2, Vector2};

/// Identifiant global représentant sur la map l'absence de tuile.
pub const EMPTY_TILE: u16 = 0;

/// Contient les données associées à une tuile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tile {
    /// Identifiant local (au sein du jeu de tuiles) de la tuile.
    pub id: u16,
    /// Chemin d'accès relatif de la texture associée à la tuile.
    pub image_path: String,
}

impl Tile {
    /// Crée une nouvelle tuile avec l'image et l'identifiant passés en paramètre.
    pub fn new(id: u16, image_path: String) -> Self {
        Tile {
            id,
            image_path
        }
    }
}

impl Default for Tile {
    fn default() -> Self {
        Tile::new(0, String::from("empty path"))
    }
}

/// Origine des tuiles d'un jeu de tuiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TilesOrigin {
    /// Les tuiles partagent la même image.
    Image(String),
    /// Chaque tuile possède sa propre image.
    Collection(BTreeMap<u16, Tile>),
    /// Les tuiles ne possèdent aucune origine.
    None,
}

impl TilesOrigin {
    /// Initialise une nouvelle collection de tuiles avec celle passée en paramètre.
    pub fn new_collection(tile: Tile) -> Self {
        let mut collection = BTreeMap::new();
        collection.insert(tile.id, tile);

        Self::Collection(collection)
    }

    /// Initialise une nouvelle collection de tuiles avec le contenu de l'itérateur
    /// passé en paramètre.
    pub fn new_collection_from<T: IntoIterator<Item=Tile>>(tiles: T) -> Self {
        let mut collection = BTreeMap::new();

        for tile in tiles {
            collection.insert(tile.id, tile);
        }

        Self::Collection(collection)
    }

    /// Insère une tuile dans la collection.
    ///
    /// Si l'origine des tuiles n'est pas une collection, alors crée une nouvelle
    /// collection et écrase l'ancienne origine.
    pub fn insert_collection(&mut self, tile: Tile) {
        match self {
            Self::Collection(tiles) => {
                tiles.insert(tile.id, tile);
            }
            _ => *self = Self::new_collection(tile),
        }
    }
}

/// Contient les paramètres d'un jeu de tuiles.
#[derive(Clone, Debug, PartialEq)]
pub struct TileSet {
    /// Identifiant global à partir duquel la tuile appartient à ce jeu.
    pub firstgid: u16,
    /// Taille en pixel des tuiles du jeu.
    pub size: Vector2<u16>,
    /// Nombre de tuiles que possède le jeu.
    pub count: u16,
    /// Nombre de colonnes que possède le jeu.
    pub columns: u16,
    /// Nom du jeu de tuile.
    pub name: String,
    /// Origine des tuiles du jeu.
    pub origin: TilesOrigin,
}

impl TileSet {
    /// Renvoie le nombre de lignes que possède le jeu.
    #[inline]
    pub fn rows(&self) -> u16 {
        self.count / self.columns
    }

    /// Renvoie le dernier identifiant global appartenant au jeu de tuiles.
    #[inline]
    pub fn last_gid(&self) -> u16 {
        self.firstgid + self.last_id()
    }

    /// Renvoie le dernier identifiant local de tuile (celui avec la plus haute
    /// valeur dans le jeu de tuiles).
    pub fn last_id(&self) -> u16 {
        match &self.origin {
            TilesOrigin::Collection(c) => *c.keys().next_back().unwrap_or(&0),
            _ if self.count > 1 => self.count - 1,
            _ => 0
        }
    }
}

impl Default for TileSet {
    fn default() -> Self {
        Self {
            firstgid: u16::MAX,
            size: Vector2 { x: 0, y: 0 },
            count: 0,
            columns: 0,
            name: String::from("unnamed"),
            origin: TilesOrigin::None,
        }
    }
}

/// Contient les données associées à un objet.
///
/// Contrairement aux tuiles, les objets possèdent l'avantages de ne pas avoir à
/// être alignés sur la grille, et peuvent donc servir à représenter diverses
/// informations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Object {
    /// Identifiant unique de l'objet.
    pub id: u16,
    /// Identifiant global de la tuile liée à l'objet.
    ///
    /// Dans le cas où aucune tuile n'est liée à l'objet, l'identifiant vaut 0.
    pub gid: u16,
    /// Coordonnées de l'objet en pixels.
    pub coords: Point2<u16>,
    /// Taille de l'objet en pixels.
    pub size: Vector2<u16>,
}

impl Object {
    /// Renvoie l'identifiant global de l'objet seulement s'il est valide.
    ///
    /// Un gid est invalide lorsqu'il est égal à 0.
    pub fn valid_gid(&self) -> Option<u16> {
        if self.gid == 0 { None } else { Some(self.gid) }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self {
            id: 0,
            gid: 0,
            coords: Point2 { x: 0, y: 0 },
            size: Vector2 { x: 0, y: 0 },
        }
    }
}

/// Contient les données associées à un groupe d'objets.
///
/// Tout comme les tuiles, les objets sont rassemblés par calques, ici appelés
/// groupes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectGroup {
    /// Identifiant unique du calque.
    pub id: u16,
    /// Nom du groupe d'objet.
    pub name: String,
    /// Liste des objets appartenant au groupe.
    pub objects: Vec<Object>,
}

impl Default for ObjectGroup {
    fn default() -> Self {
        Self {
            id: 0,
            name: String::from(""),
            objects: Vec::new(),
        }
    }
}

/// Représente les erreurs possibles lors de la conversion d'une chaîne de
/// caractère.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParsingError {
    /// La chaîne de caractère est vide.
    EmptyString,
    /// La chaîne de caractère ne peut pas être convertie dans le type souhaité.
    InvalidString(String),
}

impl ParsingError {
    /// Renvoie une chaîne de caractère décrivant l'erreur obtenue.
    fn description(&self) -> String {
        match self {
            Self::EmptyString => String::from("cannot parse from an empty string"),
            Self::InvalidString(s) => format!("this string is invalid: {}", s),
        }
    }
}

impl std::error::Error for ParsingError {}
impl fmt::Display for ParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.description().fmt(f)
    }
}

/// Représente les différentes orientations possibles pour une grille.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Orientation {
    /// Il s'agit d'une grille orthogonale.
    Orthogonal,
    /// Il s'agit d'une grille isométrique.
    Isometric,
    /// Il s'agit d'une grille possédant un décalage.
    Staggered,
    /// Il s'agit d'une grille hexagonale.
    Hexagonal
}

impl FromStr for Orientation {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "orthogonal" => Ok(Self::Orthogonal),
            "isometric" => Ok(Self::Isometric),
            "staggered" => Ok(Self::Staggered),
            "hexagonal" => Ok(Self::Hexagonal),
            "" => Err(ParsingError::EmptyString),
            _ => Err(ParsingError::InvalidString(String::from(s))),
        }
    }
}

/// Représente l'axe de décalage d'une map.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StaggerAxis {
    /// La map possède un décalage sur l'axe x.
    XAxis,
    /// La map possède un décalage sur l'axe y.
    YAxis,
    /// Aucun axe de la map n'a de décalage.
    None,
}

impl FromStr for StaggerAxis {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "x" => Ok(Self::XAxis),
            "y" => Ok(Self::YAxis),
            "" => Err(ParsingError::EmptyString),
            _ => Err(ParsingError::InvalidString(String::from(s))),
        }
    }
}

/// Contient toutes les données d'une map composée de tuiles.
#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    /// Contient un exemplaire de chaque jeu de tuiles.
    tilesets: Vec<TileSet>,
    /// Contient pour chaque id global l'indice de son jeu de tuiles.
    tileset_indexes: Vec<Option<usize>>,
    /// Taille de la map.
    pub size: Vector2<u16>,
    /// Taille en pixels des tuiles composant la map.
    pub tile_size: Vector2<u16>,
    /// Liste d'identifiants globaux des tuiles composant la map.
    pub tiles: Vec<u16>,
    /// Liste des groupes d'objets associés à la map.
    pub object_groups: Vec<ObjectGroup>,
    /// Orientation de la map.
    pub orientation: Orientation,
    /// Axe de décalage de la map.
    pub stagger_axis: StaggerAxis,
}

impl Map {
    /// Ajoute un jeu de tuiles à la fin de la liste.
    #[inline]
    pub(crate) fn add_tileset_without_reordering(&mut self, tileset: TileSet) {
        self.tilesets.push(tileset);
    }

    /// Réordonne la liste des jeux de tuiles afin qu'ils soient dans l'ordre de
    /// leur `firstgid`, puis associe pour chaque gid un jeu de tuiles.
    pub(crate) fn reorder_tilesets(&mut self) {
        self.tilesets.sort_unstable_by(|a, b| a.firstgid.cmp(&b.firstgid));

        self.tileset_indexes.clear();
        // Le gid 0 ne peut pas posséder de jeu de tuiles.
        self.tileset_indexes.push(None);

        let mut last_gid = 0;

        for (i, tileset) in self.tilesets.iter().enumerate() {
            // On comble les trous entre le nouveau et le dernier jeu de tuiles.
            for _ in last_gid..(tileset.firstgid - 1) {
                self.tileset_indexes.push(None);
            }

            // On rajoute autant d'indices vers le jeu de tuiles que de tuiles que celui-ci
            // peut contenir.

            let last_id = tileset.last_id();

            for _ in 0..=last_id {
                self.tileset_indexes.push(Some(i));
            }

            last_gid = tileset.firstgid + last_id;
        }
    }

    /// Renvoie la liste des jeux de tuiles existants.
    #[inline]
    pub fn tilesets(&self) -> &[TileSet] {
        &self.tilesets
    }

    /// Renvoie la liste mutable des jeux de tuiles existants.
    #[inline]
    pub fn tilesets_mut(&mut self) -> &mut [TileSet] {
        &mut self.tilesets
    }

    /// Ajoute un nouveau jeu de tuiles à la map.
    pub fn add_tileset(&mut self, tileset: TileSet) {
        self.add_tileset_without_reordering(tileset);
        self.reorder_tilesets();
    }

    /// Ajoute tous les jeux de tuiles de l'itérateur passé en paramètre à la map.
    pub fn add_tilesets<T: IntoIterator<Item=TileSet>>(&mut self, tilesets: T) {
        self.tilesets.extend(tilesets);
        self.reorder_tilesets();
    }

    /// Renvoie le jeu de tuiles associé au gid passé en paramètre.
    pub fn get_tileset(&self, gid: u16) -> Option<&TileSet> {
        match self.tileset_indexes.get(usize::from(gid)) {
            Some(&Some(i)) => self.tilesets.get(i),
            _ => None,
        }
    }

    /// Renvoie une référence mutable du jeu de tuiles associé au gid passé en
    /// paramètre.
    pub fn get_tileset_mut(&mut self, gid: u16) -> Option<&mut TileSet> {
        match self.tileset_indexes.get(usize::from(gid)) {
            Some(&Some(i)) => self.tilesets.get_mut(i),
            _ => None,
        }
    }

    /// Renvoie le jeu de tuiles possédant le firstgid le plus élevé.
    #[inline]
    pub fn last_tileset(&self) -> Option<&TileSet> {
        self.tilesets.last()
    }

    /// Renvoie la colonne à laquelle appartient la tuile passée en paramètre.
    #[inline]
    pub fn tile_column(&self, tile: u16) -> u16 {
        tile % self.size.x
    }

    /// Renvoie la ligne à laquelle appartient la tuile passée en paramètre.
    #[inline]
    pub fn tile_row(&self, tile: u16) -> u16 {
        tile / self.size.x
    }

    /// Renvoie l'id de la tuile appartenant aux coordonnées spécifiées.
    #[inline]
    pub fn tile_id(&self, coords: Point2<u16>) -> u16 {
        coords.x + coords.y * self.size.x
    }

    /// Renvoie l'identifiant global de la tuile appartenant aux coordonnées
    /// spécifiées.
    #[inline]
    pub fn tile_gid(&self, coords: Point2<u16>) -> u16 {
        *self.tiles.get(usize::from(self.tile_id(coords))).unwrap_or(&EMPTY_TILE)
    }

    /// Renvoie les coordonnées de la tuile sur la map.
    pub fn coords(&self, tile: u16) -> Point2<u16> {
        Point2 {
            x: self.tile_column(tile),
            y: self.tile_row(tile),
        }
    }

    /// Renvoie les coordonnées de la tuile dans le monde.
    ///
    /// Attention, ces coordonnées sont relatives à la position de la map dans le
    /// monde et représente le centre de la tuile.
    #[inline]
    pub fn world_coords(&self, tile: u16) -> Point2<f32> {
        self.to_world_coords(self.coords(tile))
    }

    /// Convertie les coordonnées de la map en coordonnées du monde.
    ///
    /// Attention, ces coordonnées sont relatives à la position de la map dans le
    /// monde et représente le centre de la tuile.
    pub fn to_world_coords(&self, map_coords: Point2<u16>) -> Point2<f32> {
        let size = Vector2 {
            x: f32::from(self.tile_size.x),
            y: f32::from(self.tile_size.y),
        };

        let multiplier = match (self.orientation, self.stagger_axis) {
            (Orientation::Hexagonal, StaggerAxis::XAxis) => Point2 {
                x: size.x * 0.75,
                y: size.y,
            },
            (Orientation::Hexagonal, StaggerAxis::YAxis) => Point2 {
                x: size.x,
                y: size.y * 0.75,
            },
            _ => Point2::from(size),
        };

        let mut coords = Point2 {
            x: f32::from(map_coords.x) * multiplier.x,
            y: -f32::from(map_coords.y) * multiplier.y,
        };

        match self.coords_stagger_axis(map_coords) {
            StaggerAxis::YAxis => {
                coords.y -= size.y / 2.0;
                coords.x += size.x;
            },
            StaggerAxis::XAxis => {
                coords.y -= size.y;
                coords.x += size.x / 2.0;
            },
            StaggerAxis::None => {
                coords.y -= size.y / 2.0;
                coords.x += size.x / 2.0;
            }
        }

        coords
    }

    /// Renvoie l'axe de décalage de la tuile passée en paramètre.
    #[inline]
    pub fn tile_stagger_axis(&self, tile: u16) -> StaggerAxis {
        self.coords_stagger_axis(self.coords(tile))
    }

    /// Renvoie l'axe de décalage de la tuile dont les coordonnées sont passés en
    /// paramètre.
    pub fn coords_stagger_axis(&self, coords: Point2<u16>) -> StaggerAxis {
        match self.stagger_axis {
            axis @ StaggerAxis::XAxis if coords.x % 2 == 1 => axis,
            axis @ StaggerAxis::YAxis if coords.y % 2 == 1 => axis,
            _ => StaggerAxis::None,
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self {
            tilesets: Vec::new(),
            tileset_indexes: Vec::new(),
            size: Vector2 { x: 0, y: 0 },
            tile_size: Vector2 { x: 0, y: 0 },
            tiles: Vec::new(),
            object_groups: Vec::new(),
            orientation: Orientation::Orthogonal,
            stagger_axis: StaggerAxis::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use mint::Vector2;
    use super::*;

    const TEST_SIZE: Vector2<u16> = Vector2 { x: 16, y: 16 };

    /// Vérifie que le jeu de tuiles est bien associé à ces gids.
    fn tileset_gids_association(map: &Map, tileset: &TileSet) {
        for gid in tileset.firstgid..=tileset.last_gid() {
            assert_eq!(map.get_tileset(gid), Some(tileset));
        }
    }

    #[test]
    fn tilesets_test() {
        let mut map = Map::default();

        let tileset_none = TileSet {
            firstgid: 1,
            count: 2,
            origin: TilesOrigin::None,
            ..Default::default()
        };
        let tileset_image = TileSet {
            firstgid: tileset_none.last_gid() + 1,
            count: 4,
            origin: TilesOrigin::Image(String::from("")),
            ..Default::default()
        };

        let tile = Tile::new(4, String::from(""));
        let tileset_collection = TileSet {
            firstgid: tileset_image.last_gid() + 1,
            count: 4,
            origin: TilesOrigin::new_collection(tile),
            ..Default::default()
        };

        map.add_tilesets(vec![
            tileset_none.clone(),
            tileset_image.clone(),
            tileset_collection.clone()
        ]);

        assert_eq!(map.get_tileset(0), None);

        tileset_gids_association(&map, &tileset_none);
        tileset_gids_association(&map, &tileset_image);
        tileset_gids_association(&map, &tileset_collection);

        assert_eq!(map.get_tileset(12), None);
    }

    #[test]
    fn tile_id_test() {
        let mut map = Map::default();
        map.size = TEST_SIZE;

        assert_eq!(map.tile_id([0, 0].into()), 0);
        assert_eq!(map.tile_id([3, 0].into()), 3);
        assert_eq!(map.tile_id([3, 1].into()), 19);
    }

    #[test]
    fn coords_test() {
        let mut map = Map::default();
        map.size = TEST_SIZE;

        assert_eq!(map.coords(0), [0, 0].into());
        assert_eq!(map.coords(3), [3, 0].into());
        assert_eq!(map.coords(19), [3, 1].into());
    }

    #[test]
    fn orthogonal_to_world_coords_test() {
        let mut map = Map::default();
        map.tile_size = TEST_SIZE;

        assert_eq!(map.to_world_coords([3, 1].into()), [56.0, -24.0].into());
    }

    #[test]
    fn hexagonal_to_world_coords_test() {
        let mut map = Map::default();
        map.orientation = Orientation::Hexagonal;
        map.tile_size = TEST_SIZE;

        let even_coords = [2, 2].into();
        let x_odd_coords = [3, 2].into();
        let y_odd_coords = [2, 3].into();

        map.stagger_axis = StaggerAxis::XAxis;
        assert_eq!(map.to_world_coords(even_coords), [32.0, -40.0].into());
        assert_eq!(map.to_world_coords(x_odd_coords), [44.0, -48.0].into());

        map.stagger_axis = StaggerAxis::YAxis;
        assert_eq!(map.to_world_coords(even_coords), [40.0, -32.0].into());
        assert_eq!(map.to_world_coords(y_odd_coords), [48.0, -44.0].into());
    }

    #[test]
    fn coords_stagger_axis_test() {
        let mut map = Map::default();

        assert_eq!(map.coords_stagger_axis([0, 0].into()), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis([1, 0].into()), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis([0, 1].into()), StaggerAxis::None);

        map.stagger_axis = StaggerAxis::XAxis;
        assert_eq!(map.coords_stagger_axis([0, 0].into()), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis([1, 0].into()), StaggerAxis::XAxis);
        assert_eq!(map.coords_stagger_axis([0, 1].into()), StaggerAxis::None);

        map.stagger_axis = StaggerAxis::YAxis;
        assert_eq!(map.coords_stagger_axis([0, 0].into()), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis([1, 0].into()), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis([0, 1].into()), StaggerAxis::YAxis);
    }
}