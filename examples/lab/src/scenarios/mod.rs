use bevy::{math::Vec2, prelude::*};
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};

use crate::LabControl;
use saddle_world_hex_grid::AxialHex;

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "hex_grid_smoke",
        "hex_grid_layouts",
        "hex_grid_pathfinding",
        "hex_grid_ranges",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(build_smoke("smoke_launch")),
        "hex_grid_smoke" => Some(build_smoke("hex_grid_smoke")),
        "hex_grid_layouts" => Some(hex_grid_layouts()),
        "hex_grid_pathfinding" => Some(hex_grid_pathfinding()),
        "hex_grid_ranges" => Some(hex_grid_ranges()),
        _ => None,
    }
}

fn set_control(mutator: impl Fn(&mut LabControl) + Send + Sync + 'static) -> Action {
    Action::Custom(Box::new(move |world| {
        let mut control = world.resource_mut::<LabControl>();
        mutator(&mut control);
    }))
}

fn wait_for_diagnostics(
    label: impl Into<String>,
    condition: impl Fn(&crate::LabDiagnostics) -> bool + Send + Sync + 'static,
) -> Action {
    Action::WaitUntil {
        label: label.into(),
        condition: Box::new(move |world| condition(world.resource::<crate::LabDiagnostics>())),
        max_frames: 90,
    }
}

fn build_smoke(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description("Boot the hex_grid lab, verify the layout, pathfinding, and range diagnostics all initialize, and capture the default showcase.")
        .then(wait_for_diagnostics("default diagnostics ready", |diagnostics| {
            diagnostics.path_exists
                && diagnostics.flat_hover_hex != diagnostics.pointy_hover_hex
                && diagnostics.reachable_count == 56
                && diagnostics.ring_count == 18
                && diagnostics.spiral_count == 37
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "path exists",
            |diagnostics| diagnostics.path_exists,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "flat and pointy layouts disagree for the same sample point",
            |diagnostics| diagnostics.flat_hover_hex != diagnostics.pointy_hover_hex,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "range overlay populated",
            |diagnostics| diagnostics.reachable_count > 0 && diagnostics.ring_count > 0,
        ))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary(name))
        .build()
}

fn hex_grid_layouts() -> Scenario {
    Scenario::builder("hex_grid_layouts")
        .description("Capture both the shared-origin case and an offset case to prove flat-top and pointy-top layouts map the same local point differently.")
        .then(set_control(|control| {
            control.sample_local_point = Vec2::ZERO;
        }))
        .then(wait_for_diagnostics("shared origin sample", |diagnostics| {
            diagnostics.flat_hover_hex == AxialHex::ZERO
                && diagnostics.pointy_hover_hex == AxialHex::ZERO
        }))
        .then(Action::Screenshot("layouts_origin".into()))
        .then(Action::WaitFrames(1))
        .then(set_control(|control| {
            control.sample_local_point = Vec2::new(36.0, -12.0);
        }))
        .then(wait_for_diagnostics("offset sample", |diagnostics| {
            diagnostics.flat_hover_hex == AxialHex::new(1, -1)
                && diagnostics.pointy_hover_hex == AxialHex::new(1, 0)
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "flat hover coordinate",
            |diagnostics| diagnostics.flat_hover_hex == AxialHex::new(1, -1),
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "pointy hover coordinate",
            |diagnostics| diagnostics.pointy_hover_hex == AxialHex::new(1, 0),
        ))
        .then(Action::Screenshot("layouts_offset".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("hex_grid_layouts"))
        .build()
}

fn hex_grid_pathfinding() -> Scenario {
    Scenario::builder("hex_grid_pathfinding")
        .description("Capture the default route first, then enable a reroute barrier and verify that A* still finds a longer path around the new blockers.")
        .then(set_control(|control| {
            control.path_goal = AxialHex::new(3, -2);
            control.reroute_barrier_enabled = false;
        }))
        .then(wait_for_diagnostics("default path", |diagnostics| {
            diagnostics.path_exists && diagnostics.path_len == 6 && diagnostics.path_cost == 5
        }))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("path_default".into()))
        .then(Action::WaitFrames(1))
        .then(set_control(|control| {
            control.path_goal = AxialHex::new(4, -2);
            control.reroute_barrier_enabled = true;
        }))
        .then(wait_for_diagnostics("rerouted path", |diagnostics| {
            diagnostics.path_exists && diagnostics.path_len == 8 && diagnostics.path_cost == 7
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "path remains reachable",
            |diagnostics| diagnostics.path_exists,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "path length matches rerouted route",
            |diagnostics| diagnostics.path_len == 8,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "path cost accounts for the detour",
            |diagnostics| diagnostics.path_cost == 7,
        ))
        .then(Action::Screenshot("path_reroute".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("hex_grid_pathfinding"))
        .build()
}

fn hex_grid_ranges() -> Scenario {
    Scenario::builder("hex_grid_ranges")
        .description("Capture a tight range pass first, then expand the movement budget and ring radius to prove the overlay scales correctly.")
        .then(set_control(|control| {
            control.movement_budget = 2;
            control.range_radius = 2;
        }))
        .then(wait_for_diagnostics("tight range", |diagnostics| {
            diagnostics.reachable_count == 15
                && diagnostics.ring_count == 12
                && diagnostics.spiral_count == 19
        }))
        .then(Action::Screenshot("ranges_tight".into()))
        .then(Action::WaitFrames(1))
        .then(set_control(|control| {
            control.movement_budget = 4;
            control.range_radius = 3;
        }))
        .then(wait_for_diagnostics("expanded range", |diagnostics| {
            diagnostics.reachable_count == 56
                && diagnostics.ring_count == 18
                && diagnostics.spiral_count == 37
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "ring count",
            |diagnostics| diagnostics.ring_count == 18,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "spiral count",
            |diagnostics| diagnostics.spiral_count == 37,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "reachable count",
            |diagnostics| diagnostics.reachable_count == 56,
        ))
        .then(Action::Screenshot("ranges".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("hex_grid_ranges"))
        .build()
}
