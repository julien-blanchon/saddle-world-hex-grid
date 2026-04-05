mod algorithms;
mod bounds;
mod coords;
mod debug;
mod direction;
mod fov;
mod layout;
mod pathfinding;
mod storage;
mod topology;

pub use algorithms::{
    DoubledRectangleIter, LineIter, OffsetRectangleIter, ParallelogramIter, RangeIter, RingIter,
    SpiralIter, TriangleIter, WedgeIter, doubled_rectangle, offset_rectangle, parallelogram,
    triangle, wedge,
};
pub use bounds::HexBounds;
pub use coords::{
    AxialHex, CubeHex, DoubledHex, DoubledHexMode, FractionalHex, HexInvariantError, OffsetHex,
    OffsetHexMode,
};
pub use debug::{HexDebugOverlay, HexGridDebugGizmos, HexGridDebugSettings};
pub use direction::{HexDiagonalDirection, HexDirection};
pub use fov::{DiagonalWay, directional_fov, range_fov};
pub use layout::{HexLayout, HexOrientation};
pub use pathfinding::{HexPath, HexReachability, a_star, reachable_within};
pub use storage::HexagonalMap;
pub use topology::{GridEdge, GridVertex};

use bevy::{
    app::PostStartup,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum HexGridSystems {
    SyncDebug,
    DrawDebug,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

#[derive(Default, Resource)]
struct HexGridRuntimeState {
    active: bool,
}

pub struct HexGridPlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
    pub debug_settings: HexGridDebugSettings,
}

impl HexGridPlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
            debug_settings: HexGridDebugSettings::default(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }

    pub fn with_debug_settings(mut self, debug_settings: HexGridDebugSettings) -> Self {
        self.debug_settings = debug_settings;
        self
    }
}

impl Default for HexGridPlugin {
    fn default() -> Self {
        Self::always_on(Update)
    }
}

impl Plugin for HexGridPlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        if !app.world().contains_resource::<HexGridDebugSettings>() {
            app.insert_resource(self.debug_settings.clone());
        }

        app.init_resource::<HexGridRuntimeState>()
            .init_gizmo_group::<HexGridDebugGizmos>()
            .register_type::<AxialHex>()
            .register_type::<CubeHex>()
            .register_type::<DoubledHex>()
            .register_type::<DoubledHexMode>()
            .register_type::<FractionalHex>()
            .register_type::<HexBounds>()
            .register_type::<HexDebugOverlay>()
            .register_type::<HexGridDebugSettings>()
            .register_type::<HexDirection>()
            .register_type::<HexDiagonalDirection>()
            .register_type::<HexLayout>()
            .register_type::<HexOrientation>()
            .register_type::<OffsetHex>()
            .register_type::<OffsetHexMode>()
            .register_type::<GridEdge>()
            .register_type::<GridVertex>()
            .configure_sets(
                self.update_schedule,
                (HexGridSystems::SyncDebug, HexGridSystems::DrawDebug).chain(),
            )
            .add_systems(self.activate_schedule, activate_runtime)
            .add_systems(self.deactivate_schedule, deactivate_runtime)
            .add_systems(
                self.update_schedule,
                debug::sync_debug_overlays
                    .in_set(HexGridSystems::SyncDebug)
                    .run_if(runtime_is_active),
            )
            .add_systems(
                self.update_schedule,
                draw_debug
                    .in_set(HexGridSystems::DrawDebug)
                    .run_if(runtime_is_active),
            );
    }
}

fn activate_runtime(mut runtime: ResMut<HexGridRuntimeState>) {
    runtime.active = true;
}

fn deactivate_runtime(mut runtime: ResMut<HexGridRuntimeState>) {
    runtime.active = false;
}

fn runtime_is_active(runtime: Res<HexGridRuntimeState>) -> bool {
    runtime.active
}

fn draw_debug(world: &mut World) {
    debug::draw_debug(world);
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
