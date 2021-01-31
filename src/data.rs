use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use bevy_math::{Size, Vec2};

/// Identifiant global représentant sur la map l'absence de tuile.
pub const EMPTY_TILE: u16 = 0;

/// Contient les données associées à une tuile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tile {
    /// Identifiant local (au sein du jeu de tuiles) de la tuile.
    pub id: u16,
    /// Chemin d'accès de la texture associée à la tuile.
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
pub enum TileOrigin {
    /// Les tuiles partagent la même image.
    Image(String),
    /// Chaque tuile possède sa propre image.
    Collection(BTreeMap<u16, Tile>),
    /// Les tuiles ne possèdent aucune origine.
    None,
}

impl TileOrigin {
    /// Insère une tuile dans la collection.
    ///
    /// Si l'origine de la tuile n'est pas une collection, alors crée une nouvelle
    /// collection et écrase l'ancienne origine.
    pub fn insert_collection(&mut self, tile: Tile) {
        if let Self::Collection(tiles) = self {
            tiles.insert(tile.id, tile);
        } else {
            let mut tiles = BTreeMap::new();
            tiles.insert(tile.id, tile);

            *self = Self::Collection(tiles);
        }
    }
}

/// Contient les paramètres d'un jeu de tuiles.
#[derive(Clone, Debug, PartialEq)]
pub struct TileSet {
    /// Identifiant global à partir duquel la tuile appartient à ce jeu.
    pub firstgid: u16,
    /// Taille en pixel des tuiles du jeu.
    pub size: Size<u16>,
    /// Nombre de tuiles que possède le jeu.
    pub count: u16,
    /// Nombre de colonnes que possède le jeu.
    pub columns: u16,
    /// Nom du jeu de tuile.
    pub name: String,
    /// Origine des tuiles du jeu.
    pub origin: TileOrigin,
}

impl TileSet {
    /// Renvoie le nombre de lignes que possède le jeu.
    #[inline]
    pub fn rows(&self) -> u16 {
        self.count / self.columns
    }
}

impl Default for TileSet {
    fn default() -> Self {
        Self {
            firstgid: u16::MAX,
            size: Size::new(0, 0),
            count: 0,
            columns: 0,
            name: String::from("unnamed"),
            origin: TileOrigin::None,
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
    /// Contient un unique exemplaire de chaque jeu de tuiles.
    unique_tilesets: Vec<Arc<TileSet>>,
    /// Contient pour chaque id global son jeu de tuiles.
    tilesets: Vec<Option<Arc<TileSet>>>,
    /// Taille de la map.
    pub size: Size<u16>,
    /// Taille en pixels des tuiles composant la map.
    pub tile_size: Size<u16>,
    /// Liste d'identifiants globaux des tuiles composant la map.
    pub tiles: Vec<u16>,
    /// Orientation de la map.
    pub orientation: Orientation,
    /// Axe de décalage de la map.
    pub stagger_axis: StaggerAxis,
}

impl Map {
    /// Ajoute un jeu de tuile à la fin de la liste.
    #[inline]
    pub(crate) fn add_tileset_without_reordering(&mut self, tileset: TileSet) {
        self.unique_tilesets.push(Arc::new(tileset));
    }

    /// Réordonne la liste des jeux de tuiles afin qu'ils soient dans l'ordre de
    /// leur `firstgid`.
    pub(crate) fn reorder_tilesets(&mut self) {
        self.unique_tilesets.sort_unstable_by(|a, b| a.firstgid.cmp(&b.firstgid));

        let mut last_gid = 0;
        self.tilesets.clear();

        for tileset in &self.unique_tilesets {
            // On comble les trous entre deux jeux de tuiles
            for _ in last_gid..(tileset.firstgid - 1) {
                self.tilesets.push(None);
            }

            // On récupère le nombre de tuiles que possède le jeu.
            let tiles_nb = match &tileset.origin {
                TileOrigin::Collection(tiles)
                    => *tiles.keys().next_back().unwrap_or(&0) + 1,
                TileOrigin::None | TileOrigin::Image(_) => tileset.count,
            };

            for _ in 0..tiles_nb {
                self.tilesets.push(Some(tileset.clone()));
            }

            last_gid = tileset.firstgid + tiles_nb - 1;
        }
    }

    /// Renvoie la liste des jeux de tuiles existants.
    #[inline]
    pub fn unique_tilesets(&self) -> Vec<&TileSet> {
        self.unique_tilesets.iter().map(|t| t.as_ref()).collect()
    }

    /// Ajoute un nouveau jeu de tuiles à la map.
    pub fn add_tileset(&mut self, tileset: TileSet) {
        self.add_tileset_without_reordering(tileset);
        self.reorder_tilesets();
    }

    /// Renvoie le jeu de tuiles associé au gid passé en paramètre.
    pub fn get_tileset(&self, tile_gid: u16) -> Option<&TileSet> {
        let index = usize::from(tile_gid - 1);

        if let Some(ref_tileset) = self.tilesets.get(index) {
            if let Some(tileset) = ref_tileset {
                return Some(tileset.as_ref());
            }
        }

        None
    }

    /// Renvoie le jeu de tuiles possédant le firstgid le plus élevé.
    #[inline]
    pub fn last_tileset(&self) -> Option<&TileSet> {
        self.unique_tilesets.last().map(|ptr| ptr.as_ref())
    }

    /// Renvoie la colonne à laquelle appartient la tuile passée en paramètre.
    #[inline]
    pub fn tile_column(&self, tile: u16) -> u16 {
        tile % self.size.width
    }

    /// Renvoie la ligne à laquelle appartient la tuile passée en paramètre.
    #[inline]
    pub fn tile_row(&self, tile: u16) -> u16 {
        tile / self.size.width
    }

    /// Renvoie l'id de la tuile appartenant aux coordonnées spécifiées.
    #[inline]
    pub fn tile_id(&self, coords: (u16, u16)) -> u16 {
        coords.0 + coords.1 * self.size.width
    }

    /// Renvoie l'identifiant global de la tuile appartenant aux coordonnées
    /// spécifiées.
    #[inline]
    pub fn tile_gid(&self, coords: (u16, u16)) -> u16 {
        *self.tiles.get(usize::from(self.tile_id(coords))).unwrap_or(&EMPTY_TILE)
    }

    /// Renvoie les coordonnées de la tuile sur la map.
    pub fn coords(&self, tile: u16) -> (u16, u16) {
        let x = self.tile_column(tile);
        let y = self.tile_row(tile);

        (x, y)
    }

    /// Renvoie les coordonnées de la tuile dans le monde.
    ///
    /// Attention, ces coordonnées sont relatives à la position de la map dans le
    /// monde et représente le centre de la tuile.
    #[inline]
    pub fn world_coords(&self, tile: u16) -> Vec2 {
        self.to_world_coords(self.coords(tile))
    }

    /// Convertie les coordonnées de la map en coordonnées du monde.
    ///
    /// Attention, ces coordonnées sont relatives à la position de la map dans le
    /// monde et représente le centre de la tuile.
    pub fn to_world_coords(&self, map_coords: (u16, u16)) -> Vec2 {
        let size = Size {
            width: f32::from(self.tile_size.width),
            height: f32::from(self.tile_size.height),
        };

        let multiplier = match (self.orientation, self.stagger_axis) {
            (Orientation::Hexagonal, StaggerAxis::XAxis) => Vec2 {
                x: size.width * 0.75,
                y: size.height,
            },
            (Orientation::Hexagonal, StaggerAxis::YAxis) => Vec2 {
                x: size.width,
                y: size.height * 0.75,
            },
            _ => Vec2 { x: size.width, y: size.height },
        };

        let mut coords = Vec2 {
            x: f32::from(map_coords.0) * multiplier.x,
            y: -f32::from(map_coords.1) * multiplier.y,
        };

        match self.coords_stagger_axis(map_coords) {
            StaggerAxis::YAxis => {
                coords.y -= size.height / 2.0;
                coords.x += size.width;
            },
            StaggerAxis::XAxis => {
                coords.y -= size.height;
                coords.x += size.width / 2.0;
            },
            StaggerAxis::None => {
                coords.y -= size.height / 2.0;
                coords.x += size.width / 2.0;
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
    pub fn coords_stagger_axis(&self, coords: (u16, u16)) -> StaggerAxis {
        match self.stagger_axis {
            axis @ StaggerAxis::XAxis if coords.0 % 2 == 1 => axis,
            axis @ StaggerAxis::YAxis if coords.1 % 2 == 1 => axis,
            _ => StaggerAxis::None,
        }
    }
}

impl Default for Map {
    fn default() -> Self {
        Self {
            unique_tilesets: Vec::new(),
            tilesets: Vec::new(),
            size: Size::new(0, 0),
            tile_size: Size::new(0, 0),
            tiles: Vec::new(),
            orientation: Orientation::Orthogonal,
            stagger_axis: StaggerAxis::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SIZE: Size<u16> = Size { width: 16, height: 16 };

    #[test]
    fn tile_id_test() {
        let mut map = Map::default();
        map.size = TEST_SIZE;

        assert_eq!(map.tile_id((0, 0)), 0);
        assert_eq!(map.tile_id((3, 0)), 3);
        assert_eq!(map.tile_id((3, 1)), 19);
    }

    #[test]
    fn coords_test() {
        let mut map = Map::default();
        map.size = TEST_SIZE;

        assert_eq!(map.coords(0), (0, 0));
        assert_eq!(map.coords(3), (3, 0));
        assert_eq!(map.coords(19), (3, 1));
    }

    #[test]
    fn orthogonal_to_world_coords_test() {
        let mut map = Map::default();
        map.tile_size = TEST_SIZE;

        assert_eq!(map.to_world_coords((3, 1)), Vec2::new(56.0, -24.0));
    }

    #[test]
    fn hexagonal_to_world_coords_test() {
        let mut map = Map::default();
        map.orientation = Orientation::Hexagonal;
        map.tile_size = TEST_SIZE;

        let even_coords = (2, 2);
        let x_odd_coords = (3, 2);
        let y_odd_coords = (2, 3);

        map.stagger_axis = StaggerAxis::XAxis;
        assert_eq!(map.to_world_coords(even_coords), Vec2::new(32.0, -40.0));
        assert_eq!(map.to_world_coords(x_odd_coords), Vec2::new(44.0, -48.0));

        map.stagger_axis = StaggerAxis::YAxis;
        assert_eq!(map.to_world_coords(even_coords), Vec2::new(40.0, -32.0));
        assert_eq!(map.to_world_coords(y_odd_coords), Vec2::new(48.0, -44.0));
    }

    #[test]
    fn coords_stagger_axis_test() {
        let mut map = Map::default();

        assert_eq!(map.coords_stagger_axis((0, 0)), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis((1, 0)), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis((0, 1)), StaggerAxis::None);

        map.stagger_axis = StaggerAxis::XAxis;
        assert_eq!(map.coords_stagger_axis((0, 0)), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis((1, 0)), StaggerAxis::XAxis);
        assert_eq!(map.coords_stagger_axis((0, 1)), StaggerAxis::None);

        map.stagger_axis = StaggerAxis::YAxis;
        assert_eq!(map.coords_stagger_axis((0, 0)), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis((1, 0)), StaggerAxis::None);
        assert_eq!(map.coords_stagger_axis((0, 1)), StaggerAxis::YAxis);
    }
}