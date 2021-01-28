use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use bevy_math::{Size, Vec2};

/// Identifiant global représentant sur la map l'absence de tuile.
pub const EMPTY_TILE: u32 = 0;

/// Contient les données d'une tuile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tile {
    /// Identifiant local (au sein du tileset) de la tuile.
    pub id: u32,
    /// Chemin d'accès de la texture associée à la tuile.
    pub image_path: String,
}

impl Tile {
    /// Crée une nouvelle tuile avec l'image et l'identifiant passés en paramètre.
    pub fn new(id: u32, image_path: String) -> Self {
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

/// Représente l'origine des tuiles du jeu de tuiles.
#[derive(Clone, Debug)]
pub enum TileOrigin {
    /// Les tuiles partagent la même image.
    Image(String),
    /// Chaque tuile possède sa propre image.
    Collection(BTreeMap<u32, Tile>),
    /// Les tuiles ne possèdent aucune origine.
    None,
}

impl TileOrigin {
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
#[derive(Clone, Debug)]
pub struct TileSet {
    /// Identifiant global à partir du quel la tuile appartient à ce jeu.
    pub firstgid: u32,
    /// Taille en pixel des tuiles du jeu.
    pub size: Size<u32>,
    /// Nombre de tuiles que possède le jeu.
    pub count: usize,
    /// Nombre de tuiles présentes sur une colonne.
    pub columns: usize,
    /// Nom du jeu de tuile.
    pub name: String,
    /// Origine des tuiles du jeu.
    pub origin: TileOrigin,
}

impl TileSet {
    /// Renvoie le nombre de tuiles présentes sur une ligne.
    #[inline]
    pub fn rows(&self) -> usize {
        self.count / self.columns
    }
}

impl Default for TileSet {
    fn default() -> Self {
        Self {
            firstgid: u32::MAX,
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
#[derive(Clone, Debug)]
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
        match s {
            "orthogonal" => Ok(Self::Orthogonal),
            "isometric" => Ok(Self::Isometric),
            "staggered" => Ok(Self::Staggered),
            "Hexagonal" => Ok(Self::Hexagonal),
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
        match s {
            "x" => Ok(Self::XAxis),
            "y" => Ok(Self::XAxis),
            "" => Err(ParsingError::EmptyString),
            _ => Err(ParsingError::InvalidString(String::from(s))),
        }
    }
}

/// Contient toutes les informations liées à une map composée de tuiles.
#[derive(Clone, Debug)]
pub struct Map {
    unique_tilesets: Vec<Arc<TileSet>>,
    tilesets: Vec<Option<Arc<TileSet>>>,
    /// Taille de la map.
    pub size: Size<u32>,
    /// Taille des tuiles en pixels composant la map.
    pub tile_size: Size<u32>,
    /// Liste des gids des tuiles composant la map.
    pub tiles: Vec<u32>,
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

    /// Réordonne la liste des jeux de tuiles afin qu'ils soient dans l'ordre.
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
                TileOrigin::Collection(tiles) => {
                    *tiles.keys().next_back().unwrap_or(&0) as usize + 1
                },
                TileOrigin::None | TileOrigin::Image(_) => tileset.count,
            };

            for _ in 0..tiles_nb {
                self.tilesets.push(Some(tileset.clone()));
            }

            last_gid = tileset.firstgid + tileset.count as u32 - 1;
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
    pub fn get_tileset(&self, tile_gid: u32) -> Option<&TileSet> {
        let index = (tile_gid - 1) as usize;

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
    pub fn tile_column(&self, tile: u32) -> u32 {
        tile % self.size.width
    }

    /// Renvoie la ligne à laquelle appartient la tuile passée en paramètre.
    #[inline]
    pub fn tile_row(&self, tile: u32) -> u32 {
        tile / self.size.width
    }

    /// Renvoie les coordonnées de la tuile sur la map.
    pub fn coords(&self, tile: u32) -> (u32, u32) {
        let x = self.tile_column(tile);
        let y = self.tile_row(tile);

        (x, y)
    }

    /// Renvoie les coordonnées de la tuile dans le monde.
    ///
    /// Attention, ces coordonnées sont relatives à la position de la map dans le
    /// monde et représente le centre de la tuile.
    pub fn world_coords(&self, tile: u32) -> Vec2 {
        let map_coords = self.coords(tile);
        let width = self.tile_size.width as f32;
        let height = self.tile_size.height as f32;

        let mut coords = Vec2 {
            x: map_coords.0 as f32 * width,
            y: map_coords.1 as f32 * -(height),
        };
        match self.tile_stagger_axis(tile) {
            StaggerAxis::YAxis => coords.y += height / 2.0,
            StaggerAxis::XAxis => coords.x += width / 2.0,
            _ => {
                coords.y += height / 2.0;
                coords.x += width / 2.0;
            }
        }

        coords
    }

    /// Renvoie l'axe de décalage de la tuile passée en paramètre.
    pub fn tile_stagger_axis(&self, tile: u32) -> StaggerAxis {
        match self.stagger_axis {
            axis @ StaggerAxis::XAxis if self.tile_column(tile) % 2 == 0 => axis,
            axis @ StaggerAxis::YAxis if self.tile_row(tile) % 2 == 0 => axis,
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