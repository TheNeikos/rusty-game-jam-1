use std::time::Duration;

use bevy::{core::FixedTimestep, prelude::*};

use crate::{ldtk_map::LdtkMap, map::CurrentMap, markers::Markers, MainLdtk, GRID_SIZE};

pub struct MovementPlugin;

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Ord, StageLabel, Clone)]
pub enum MovementStages {
    Movement,
    PostMovement,
}

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            CoreStage::Update,
            MovementStages::Movement,
            SystemStage::parallel().with_run_criteria(FixedTimestep::steps_per_second(30.)),
        );
        app.add_stage_after(
            MovementStages::Movement,
            MovementStages::PostMovement,
            SystemStage::single_threaded(),
        )
        .add_system_to_stage(MovementStages::PostMovement, apply_speed)
        .add_system_to_stage(MovementStages::PostMovement, synchronize_to_transform);
    }
}

#[derive(Debug)]
pub struct LevelBounds(pub IVec2);

#[derive(Debug, Default)]
pub struct Gravity(pub Vec2);

#[derive(Debug)]
pub struct Speed {
    pub speed: Vec2,
    pub friction: f32,
}

impl Default for Speed {
    fn default() -> Self {
        Self {
            speed: Default::default(),
            friction: 0.85,
        }
    }
}

impl Speed {
    pub fn total_speed(&self) -> Vec2 {
        self.speed
    }

    pub fn apply_friction(&mut self) {
        self.speed *= self.friction;

        if self.speed.x.abs() < 0.0005 {
            self.speed.x = 0.;
        }

        if self.speed.y.abs() < 0.0005 {
            self.speed.y = 0.;
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub cell: IVec2,
    pub fraction: Vec2,
}

impl From<Vec2> for Position {
    fn from(fraction: Vec2) -> Self {
        let mut pos = Position {
            cell: Default::default(),
            fraction,
        };
        pos.normalize();
        pos
    }
}

impl Position {
    pub fn total_position(&self) -> Vec2 {
        self.cell.as_f32() + self.fraction
    }

    pub fn normalize(&mut self) {
        self.cell += self.fraction.floor().as_i32();
        self.fraction = self.fraction.fract();
    }
}

#[derive(Debug)]
pub struct OnGround;

fn apply_speed(
    current_map: Res<CurrentMap>,
    ldtk_assets: Res<Assets<MainLdtk>>,
    mut movement_query: Query<(&mut Position, &mut Speed, Option<&Gravity>, &mut Markers)>,
) {
    let ldtk = if let Some(LdtkMap { ldtk }) = ldtk_assets.get(&current_map.0) {
        ldtk
    } else {
        return;
    };

    for (mut position, mut speed, gravity, mut markers) in movement_query.iter_mut() {
        let level = if let Some(level) = ldtk.levels.iter().find(|level| {
            let level_bl = level.world_position_px / GRID_SIZE;
            let level_tr = level_bl + level.dimensions_px / GRID_SIZE;
            position.cell.ge(&level_bl) && position.cell.lt(&level_tr)
        }) {
            level
        } else {
            error!("Could not find associated level for entity.");
            continue;
        };

        let col_layer = &level.layers.foreground;
        let collision = match &col_layer.special {
            bevy_spicy_ldtk::SpecialValues::IntGrid { values, .. } => values,
            _ => return,
        };

        let has_collision = |mut pos: IVec2| -> bool {
            pos -= level.world_position_px / GRID_SIZE;
            let idx = pos.y * col_layer.dimensions_cell.x as i32 + pos.x;

            if pos.x >= col_layer.dimensions_cell.x as i32
                || pos.x < 0
                || pos.y >= col_layer.dimensions_cell.y as i32
                || pos.y < 0
            {
                return true;
            }

            collision[idx as usize] == 1
        };

        if let Some(gravity) = gravity.as_ref() {
            if !markers.contains::<OnGround>() {
                speed.speed += gravity.0;
            }
        }

        let steps = {
            let abs_speed = speed.total_speed().abs();

            (abs_speed.x + abs_speed.y * 5.).ceil()
        } as u32;

        // info!("Pos was: {}", position.total_position());
        for _ in 0..steps {
            position.fraction += speed.total_speed() / steps as f32;

            if speed.total_speed().x != 0. {
                // Check for x collision
                if has_collision(position.cell + IVec2::X) && position.fraction.x >= 0.8 {
                    position.fraction.x = 0.8;
                    speed.speed.x = 0.;
                }

                if has_collision(position.cell - IVec2::X) && position.fraction.x <= 0.2 {
                    position.fraction.x = 0.2;
                    speed.speed.x = 0.;
                }
            }

            if speed.total_speed().y != 0. {
                // Check for y collision
                // Ground
                if has_collision(position.cell - IVec2::Y) && position.fraction.y <= 0.0 {
                    position.fraction.y = 0.0;
                    speed.speed.y = 0.;

                    markers.add_marker_for::<OnGround>(Duration::from_millis(50));
                }

                // Top
                if has_collision(position.cell + IVec2::Y) && position.fraction.y >= 0.2 {
                    position.fraction.y = 0.2;
                    speed.speed.y = 0.;
                }
            }

            position.normalize();
            // info!("Player position: {:?}", position);
        }

        speed.apply_friction();
    }
}

fn synchronize_to_transform(
    mut movement_query: Query<(&mut Transform, &Position), Changed<Position>>,
) {
    for (mut transform, position) in movement_query.iter_mut() {
        transform.translation = (position.total_position() * crate::GRID_SIZE as f32)
            .round()
            .extend(transform.translation.z);
    }
}
