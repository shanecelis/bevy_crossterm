use bevy::utils::BoxedFuture;
use bevy_asset::io::Reader;
use bevy_asset::AsyncReadExt;
use bevy_asset::{AssetLoader, LoadContext};
use thiserror::Error;

use crate::components::{Sprite, StyleMap};

#[derive(Error, Debug)]
pub enum LoadSpriteError {
    #[error("sprite data contains invalid utf8 data")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub struct SpriteLoader;

impl AssetLoader for SpriteLoader {
    type Asset = Sprite;
    type Settings = ();
    type Error = LoadSpriteError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, LoadSpriteError>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let string = std::str::from_utf8(&bytes)?;
            let sprite = Sprite::new(string);
            Ok(sprite)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

#[derive(Error, Debug)]
pub enum LoadStyleMapError {
    #[error("error deserializing style map from ron data")]
    Deserialize(#[from] ron::de::SpannedError),
    #[error("io error")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub struct StyleMapLoader;

impl AssetLoader for StyleMapLoader {
    type Asset = StyleMap;
    type Settings = ();
    type Error = LoadStyleMapError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, LoadStyleMapError>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let stylemap = ron::de::from_bytes::<StyleMap>(&bytes)?;
            Ok(stylemap)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["stylemap"]
    }
}
