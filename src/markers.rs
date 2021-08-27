use std::{
    any::{Any, TypeId},
    time::Duration,
};

use bevy::{
    prelude::*,
    utils::{HashMap, Instant},
};

pub struct MarkerPlugin;

impl Plugin for MarkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_markers);
    }
}

fn update_markers(mut marker_query: Query<&mut Markers>) {
    for mut markers in marker_query.iter_mut() {
        markers.update();
    }
}

#[derive(Debug)]
struct MarkerInfo {
    created_at: Instant,
    destroy_after: Option<Duration>,
}

#[derive(Debug, Default)]
pub struct Markers {
    map: HashMap<TypeId, MarkerInfo>,
}

impl Markers {
    pub fn add_marker_for<T: Any>(&mut self, destroy_after: Duration) {
        self.add(std::any::TypeId::of::<T>(), Some(destroy_after));
    }

    pub fn contains<T: Any>(&self) -> bool {
        self.map.contains_key(&std::any::TypeId::of::<T>())
    }

    fn add(&mut self, marker: TypeId, destroy_after: Option<Duration>) {
        self.map.insert(
            marker,
            MarkerInfo {
                created_at: Instant::now(),
                destroy_after,
            },
        );
    }

    fn update(&mut self) {
        self.map.retain(|_, info| {
            if let Some(destroy_after) = info.destroy_after.as_ref() {
                info.created_at + *destroy_after > Instant::now()
            } else {
                true
            }
        });
    }
}
