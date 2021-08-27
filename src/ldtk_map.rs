use std::marker::PhantomData;

use bevy::asset::{AssetLoader, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;

#[derive(Debug)]
pub struct LdtkPlugin<T>(PhantomData<T>);

impl<T> Default for LdtkPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: bevy_spicy_ldtk::DeserializeLdtk + Send + Sync + 'static> Plugin for LdtkPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_asset::<LdtkMap<T>>()
            .add_asset_loader(LdtkLoader::<T>::default());
    }
}

#[derive(Debug)]
pub struct LdtkLoader<T: bevy_spicy_ldtk::DeserializeLdtk>(PhantomData<T>);

impl<T: bevy_spicy_ldtk::DeserializeLdtk> Default for LdtkLoader<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

#[derive(Debug)]
pub struct LdtkMap<T: bevy_spicy_ldtk::DeserializeLdtk> {
    pub ldtk: T,
}

impl<T: bevy_spicy_ldtk::DeserializeLdtk> TypeUuid for LdtkMap<T> {
    const TYPE_UUID: bevy::reflect::Uuid = bevy::reflect::Uuid::from_bytes([
        0xdb, 0x65, 0xd5, 0xb4, 0xaf, 0x96, 0x43, 0xbb, 0x87, 0x94, 0xce, 0x1c, 0xe1, 0xf3, 0x20,
        0x02,
    ]);
}

impl<T: bevy_spicy_ldtk::DeserializeLdtk + Send + Sync + 'static> AssetLoader for LdtkLoader<T> {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            info!("Loading ldtk from {:?}", load_context.path());

            let ldtk = ldtk2::Coordinate::from_str(String::from_utf8(bytes.to_vec())?)
                .map_err(|err| anyhow::anyhow!(err.to_string()))?;

            let ldtk = T::deserialize_ldtk(&ldtk)?;

            load_context.set_default_asset(LoadedAsset::new(LdtkMap { ldtk }));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtk"]
    }
}
