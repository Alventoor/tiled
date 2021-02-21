use std::collections::HashMap;
use std::fmt::{self, Formatter};

use serde::{Deserialize, Deserializer};
use serde::de::{MapAccess, Visitor};

use crate::data::{Image, Map, Object, Tile, TileSet, TilesOrigin};

pub use quick_xml::DeError as TMXError;

trait MapAccessExt<'de>: MapAccess<'de> {
    /// Tente d'enregistrer la valeur suivante contenue dans la table dans la
    /// variable passée en paramètre.
    ///
    /// Si la valeur ne peut être convertie dans le type de la variable, un message
    /// d'erreur est affiché, rappelant la clef associée.
    fn save_value<F>(&mut self, var: &mut F, key: &str)
    where F: Deserialize<'de>
    {
        match self.next_value::<F>() {
            Ok(value) => *var = value,
            Err(e) => eprintln!("Warning: field \"{}\" - {}", key, e),
        }
    }
}

impl<'de, T> MapAccessExt<'de> for T where T: MapAccess<'de> {}

struct ImageVisitor;

impl<'de> Visitor<'de> for ImageVisitor {
    type Value = Image;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "a tiled image")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de>
    {
        let mut image = Image::default();

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "source" => map.save_value(&mut image.source, &key),
                "width" => map.save_value(&mut image.size.x, &key),
                "height" => map.save_value(&mut image.size.y, &key),
                _ => { let _ = map.next_value::<()>(); } // Passe à la valeur suivante
            }
        }
        
        Ok(image)
    }
}

impl<'de> Deserialize<'de> for Image {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        deserializer.deserialize_map(ImageVisitor)
    }
}

struct ObjectVisitor;

impl<'de> Visitor<'de> for ObjectVisitor {
    type Value = Object;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "a tiled object")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de>
    {
        let mut object = Object::default();

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "id" => map.save_value(&mut object.id, &key),
                "gid" => map.save_value(&mut object.gid, &key),
                "x" => map.save_value(&mut object.coords.x, &key),
                "y" => map.save_value(&mut object.coords.y, &key),
                "width" => map.save_value(&mut object.size.x, &key),
                "height" => map.save_value(&mut object.size.y, &key),
                _ => { let _ = map.next_value::<()>(); } // Passe à la valeur suivante
            }
        }

        Ok(object)
    }
}

impl<'de> Deserialize<'de> for Object {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        deserializer.deserialize_map(ObjectVisitor)
    }
}

struct TileSetVisitor;

impl<'de> Visitor<'de> for TileSetVisitor {
    type Value = TileSet;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "a tiled tileset")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de>
    {
        let mut tileset = TileSet::default();

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "firstgid" => map.save_value(&mut tileset.firstgid, &key),
                "name" => map.save_value(&mut tileset.name, &key),
                "tilewidth" => map.save_value(&mut tileset.size.x, &key),
                "tileheight" => map.save_value(&mut tileset.size.y, &key),
                "tilecount" => map.save_value(&mut tileset.count, &key),
                "columns" => map.save_value(&mut tileset.columns, &key),
                "image" if tileset.origin.is_none() => {
                    if let Ok(image) = map.next_value::<Image>() {
                        tileset.origin = Some(TilesOrigin::Image(image));
                    }
                }
                "tile" if tileset.origin.is_none() => {
                    if let Ok(tiles) = map.next_value::<Vec<Tile>>() {
                        tileset.origin = Some(TilesOrigin::new_collection_from(tiles));
                    }
                }
                "image" | "tile" => {
                    println!("Warning: the tileset \"{}\" as already an origin", tileset.name);
                }
                _ => { let _ = map.next_value::<()>(); } // Passe à la valeur suivante
            }
        }

        Ok(tileset)
    }
}

impl<'de> Deserialize<'de> for TileSet {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        deserializer.deserialize_map(TileSetVisitor)
    }
}

struct MapVisitor;

impl<'de> Visitor<'de> for MapVisitor {
    type Value = Map;

    #[inline]
    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "a tiled map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where A: MapAccess<'de>
    {
        let mut tmx_map = Map::default();

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "orientation" => map.save_value(&mut tmx_map.orientation, &key),
                "width" => map.save_value(&mut tmx_map.size.x, &key),
                "height" => map.save_value(&mut tmx_map.size.y, &key),
                "tilewidth" => map.save_value(&mut tmx_map.tile_size.x, &key),
                "tileheight" => map.save_value(&mut tmx_map.tile_size.y, &key),
                "staggeraxis" => map.save_value(&mut tmx_map.stagger_axis, &key),
                "tileset" => map.save_value(&mut tmx_map.tilesets, &key),
                "objectgroup" => map.save_value(&mut tmx_map.object_groups, &key),
                "layer" => if let Ok(layer) = map.next_value::<HashMap<String, String>>() {
                    if let Some(encoded_data) = layer.get("data") {
                        tmx_map.tiles = decode_csv_data(encoded_data);
                    }
                }
                _ => { let _ = map.next_value::<()>(); } // Passe à la valeur suivante
            }
        }

        tmx_map.reorder_tilesets();

        Ok(tmx_map)
    }
}

impl<'de> Deserialize<'de> for Map {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>
    {
        deserializer.deserialize_map(MapVisitor)
    }
}

/// Décode le texte contenu au format csv dans la balise `<data>` en une liste
/// d'identifiants globaux de tuiles.
fn decode_csv_data(data: &str) -> Vec<u16> {
    data
        .split(|c: char| c == ',' || c == '\n')
        .filter_map(|d| d.trim().parse().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use mint::{Point2, Vector2};
    use serde_test::{assert_de_tokens, Token};

    use crate::data::{Image, Map, Object, ObjectGroup, Orientation, StaggerAxis,
                      Tile, TileSet, TilesOrigin};
    use super::decode_csv_data;

    #[test]
    fn test_decode_csv_data() {
        let src = r#"
        0,0,0,
        3,2,1"#;

        let should_be = vec! [0, 0, 0, 3, 2, 1];

        let tiles = decode_csv_data(src);
        assert_eq!(tiles, should_be);
    }

    #[test]
    fn test_de_image() {
        let image = Image {
            source: "foo".to_string(),
            size: Vector2 { x: 48, y: 24 },
        };

        assert_de_tokens(&image, &[
            Token::Struct { name: "image", len: 3 },
            Token::Str("source"),
            Token::String("foo"),
            Token::Str("width"),
            Token::U16(48),
            Token::Str("height"),
            Token::U16(24),
            Token::StructEnd
        ]);
    }

    #[test]
    fn test_de_tile() {
        let image = Image {
            source: "foo".to_string(),
            size: Vector2 { x: 48, y: 24 },
        };

        let tile = Tile {
            id: 0,
            image,
        };

        assert_de_tokens(&tile, &[
            Token::Struct { name: "tile", len: 2 },
            Token::Str("id"),
            Token::U16(0),
            Token::Str("image"),
            Token::Struct { name: "image", len: 3 },
            Token::Str("source"),
            Token::String("foo"),
            Token::Str("width"),
            Token::U16(48),
            Token::Str("height"),
            Token::U16(24),
            Token::StructEnd,
            Token::StructEnd,
        ]);
    }

    #[test]
    fn test_de_tileset() {
        let mut tileset = TileSet {
            firstgid: 1,
            size: Vector2 { x: 24, y: 24 },
            count: 4,
            columns: 2,
            name: "foo".to_string(),
            origin: None
        };

        let tokens = vec![
            Token::Struct { name: "tileset", len: 5 },
            Token::Str("firstgid"),
            Token::U16(1),
            Token::Str("name"),
            Token::String("foo"),
            Token::Str("tilewidth"),
            Token::U16(24),
            Token::Str("tileheight"),
            Token::U16(24),
            Token::Str("tilecount"),
            Token::U16(4),
            Token::Str("columns"),
            Token::U16(2),
            Token::StructEnd
        ];

        assert_de_tokens(&tileset, &tokens);

        let image = Image {
            source: "bar".to_string(),
            size: Vector2 { x: 24, y: 24 },
        };

        tileset.origin = Some(TilesOrigin::Image(image.clone()));

        let image_tokens = vec![
            Token::Struct { name: "tileset", len: 5 },
            Token::Str("firstgid"),
            Token::U16(1),
            Token::Str("name"),
            Token::String("foo"),
            Token::Str("tilewidth"),
            Token::U16(24),
            Token::Str("tileheight"),
            Token::U16(24),
            Token::Str("tilecount"),
            Token::U16(4),
            Token::Str("columns"),
            Token::U16(2),
            Token::Str("image"),
            Token::Struct {name: "image", len: 3 },
            Token::Str("source"),
            Token::String("bar"),
            Token::Str("width"),
            Token::U16(24),
            Token::Str("height"),
            Token::U16(24),
            Token::StructEnd,
            Token::StructEnd,
        ];

        assert_de_tokens(&tileset, &image_tokens);

        tileset.origin = Some(TilesOrigin::new_collection(Tile::new(0, image)));

        let collection_token = vec![
            Token::Struct { name: "tileset", len: 5 },
            Token::Str("firstgid"),
            Token::U16(1),
            Token::Str("name"),
            Token::String("foo"),
            Token::Str("tilewidth"),
            Token::U16(24),
            Token::Str("tileheight"),
            Token::U16(24),
            Token::Str("tilecount"),
            Token::U16(4),
            Token::Str("columns"),
            Token::U16(2),
            Token::Str("tile"),
            Token::Seq { len: Some(1) },
            Token::Struct {name: "tile", len: 2 },
            Token::Str("id"),
            Token::U16(0),
            Token::Str("image"),
            Token::Struct {name: "image", len: 3 },
            Token::Str("source"),
            Token::String("bar"),
            Token::Str("width"),
            Token::U16(24),
            Token::Str("height"),
            Token::U16(24),
            Token::StructEnd,
            Token::StructEnd,
            Token::SeqEnd,
            Token::StructEnd,
        ];

        assert_de_tokens(&tileset, &collection_token);
    }

    #[test]
    fn test_de_object() {
        let object = Object {
            id: 0,
            gid: 1,
            coords: Point2 { x: 10, y: 20 },
            size: Vector2 { x: 24, y: 12 }
        };

        assert_de_tokens(&object, &[
            Token::Struct { name: "object", len: 6 },
            Token::Str("id"),
            Token::U16(0),
            Token::Str("gid"),
            Token::U16(1),
            Token::Str("x"),
            Token::U16(10),
            Token::Str("y"),
            Token::U16(20),
            Token::Str("width"),
            Token::U16(24),
            Token::Str("height"),
            Token::U16(12),
            Token::StructEnd
        ]);
    }

    #[test]
    fn test_de_object_group() {
        let object_group = ObjectGroup {
            id: 1,
            name: "foo".to_string(),
            objects: vec![Object {
                id: 0,
                gid: 1,
                coords: Point2 { x: 10, y: 20 },
                size: Vector2 { x: 24, y: 12 }
            }]
        };

        assert_de_tokens(&object_group, &[
            Token::Struct { name: "objectgroup", len: 6 },
            Token::Str("id"),
            Token::U16(1),
            Token::Str("name"),
            Token::String("foo"),
            Token::Str("object"),
            Token::Seq { len: Some(1) },
            Token::Struct { name: "object", len: 6 },
            Token::Str("id"),
            Token::U16(0),
            Token::Str("gid"),
            Token::U16(1),
            Token::Str("x"),
            Token::U16(10),
            Token::Str("y"),
            Token::U16(20),
            Token::Str("width"),
            Token::U16(24),
            Token::Str("height"),
            Token::U16(12),
            Token::StructEnd,
            Token::SeqEnd,
            Token::StructEnd
        ]);
    }

    #[test]
    fn test_de_map() {
        let map = Map {
            tilesets: vec![],
            tileset_indexes: vec![None],
            size: Vector2 { x: 10, y: 10 },
            tile_size: Vector2 { x: 24, y: 12},
            tiles: vec![0, 0, 0, 3, 2, 1],
            object_groups: vec![],
            orientation: Orientation::Isometric,
            stagger_axis: StaggerAxis::XAxis
        };

        assert_de_tokens(&map, &[
            Token::Struct { name: "map", len: 10 },
            Token::Str("tileset"),
            Token::Seq { len: None },
            Token::SeqEnd,
            Token::Str("width"),
            Token::U16(10),
            Token::Str("height"),
            Token::U16(10),
            Token::Str("tilewidth"),
            Token::U16(24),
            Token::Str("tileheight"),
            Token::U16(12),
            Token::Str("layer"),
            Token::Map { len: Some(1) },
            Token::Str("data"),
            Token::String("0,0,0,3,2,1"),
            Token::MapEnd,
            Token::Str("objectgroup"),
            Token::Seq { len: None },
            Token::SeqEnd,
            Token::Str("orientation"),
            Token::Enum { name: "Orientation" },
            Token::Str("isometric"),
            Token::Unit,
            Token::Str("staggeraxis"),
            Token::Enum { name: "StaggerAxis" },
            Token::Str("x"),
            Token::Unit,
            Token::StructEnd
        ]);
    }
}