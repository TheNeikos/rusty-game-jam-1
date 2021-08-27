use bevy::{math::Vec3Swizzles, prelude::*, render::camera::Camera};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(camera_follow);
    }
}


#[derive(Debug, Default)]
pub struct CameraFollow;

#[derive(Debug, Default)]
pub struct ViewCamera;

fn camera_follow(
    mut camera_query: Query<&mut Transform, (With<Camera>, With<ViewCamera>)>,
    follow_query: Query<&Transform, (Without<Camera>, With<CameraFollow>, Changed<Transform>)>,
) {
    let follow_trans = match follow_query.single() {
        Ok(follow) => follow,
        Err(err) => match err {
            bevy::ecs::system::QuerySingleError::NoEntities(_) => return,
            bevy::ecs::system::QuerySingleError::MultipleEntities(_) => {
                error!("More than one entity has a `CameraFollow` component. This cannot work.");
                return;
            }
        },
    };

    let mut camera_trans = if let Ok(camera) = camera_query.single_mut() {
        camera
    } else {
        error!("More than one camera have been added. This cannot work.");
        return;
    };

    camera_trans.translation = follow_trans
        .translation
        .xy()
        .floor()
        .extend(camera_trans.translation.z);
}
