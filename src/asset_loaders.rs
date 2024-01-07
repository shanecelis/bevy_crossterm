use bevy::utils::BoxedFuture;
use bevy_asset::io::Reader;
use bevy_asset::AsyncReadExt;
use bevy_asset::{AssetLoader, LoadContext};

use crate::components::{Sprite, StyleMap};

#[derive(Default)]
pub struct SpriteLoader;

// TODO
// Library should not use anyhow

impl AssetLoader for SpriteLoader {
    type Asset = Sprite;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, anyhow::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let string = std::str::from_utf8(&bytes);
            let sprite = Sprite::new(string?);
            Ok(sprite)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["txt"]
    }
}

#[derive(Default)]
pub struct StyleMapLoader;

impl AssetLoader for StyleMapLoader {
    type Asset = StyleMap;
    type Settings = ();
    type Error = anyhow::Error;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, anyhow::Error>> {
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
