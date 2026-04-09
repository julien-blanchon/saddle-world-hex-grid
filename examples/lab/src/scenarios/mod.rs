use bevy::{math::Vec2, prelude::*};
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};

use crate::LabControl;
use saddle_world_hex_grid::{AxialHex, HexDirection, HexagonalMap, directional_fov, range_fov};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "hex_grid_smoke",
        "hex_grid_basic",
        "hex_grid_layouts",
        "hex_grid_pathfinding",
        "hex_grid_ranges",
        "hex_grid_fov",
        "hex_grid_strategy",
        "hex_grid_storage",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(build_smoke("smoke_launch")),
        "hex_grid_smoke" => Some(build_smoke("hex_grid_smoke")),
        "hex_grid_basic" => Some(hex_grid_basic()),
        "hex_grid_layouts" => Some(hex_grid_layouts()),
        "hex_grid_pathfinding" => Some(hex_grid_pathfinding()),
        "hex_grid_ranges" => Some(hex_grid_ranges()),
        "hex_grid_fov" => Some(hex_grid_fov()),
        "hex_grid_strategy" => Some(hex_grid_strategy()),
        "hex_grid_storage" => Some(hex_grid_storage()),
        _ => None,
    }
}

fn set_control(mutator: impl Fn(&mut LabControl) + Send + Sync + 'static) -> Action {
    Action::Custom(Box::new(move |world| {
        let mut control = world.resource_mut::<LabControl>();
        mutator(&mut control);
        let snapshot = control.clone();
        drop(control);

        world.resource_mut::<crate::E2EControlOverride>().0 = Some(snapshot.clone());

        let mut pane = world.resource_mut::<crate::LabPane>();
        pane.sample_x = snapshot.sample_local_point.x;
        pane.sample_y = snapshot.sample_local_point.y;
        pane.goal_q = snapshot.path_goal.q as f32;
        pane.goal_r = snapshot.path_goal.r as f32;
        pane.reroute_barrier_enabled = snapshot.reroute_barrier_enabled;
        pane.movement_budget = snapshot.movement_budget as f32;
        pane.range_radius = snapshot.range_radius as f32;
        pane.show_attack_range = snapshot.show_attack_range;
        pane.fov_range = snapshot.fov_range as f32;
        pane.fov_directional_mode = snapshot.fov_directional_mode;
        pane.fov_facing_index = snapshot.fov_facing.index() as f32;
    }))
}

fn set_sample_point(point: Vec2) -> Action {
    set_control(move |control| {
        control.sample_local_point = point;
    })
}

fn set_path_goal(goal: AxialHex, reroute_barrier_enabled: bool) -> Action {
    set_control(move |control| {
        control.path_goal = goal;
        control.reroute_barrier_enabled = reroute_barrier_enabled;
    })
}

fn set_range_mode(movement_budget: u32, range_radius: u32) -> Action {
    set_control(move |control| {
        control.movement_budget = movement_budget;
        control.range_radius = range_radius;
    })
}

fn set_fov_mode(viewer: AxialHex, range: u32, directional_mode: bool, facing: HexDirection) -> Action {
    set_control(move |control| {
        control.fov_viewer = viewer;
        control.fov_range = range;
        control.fov_directional_mode = directional_mode;
        control.fov_facing = facing;
    })
}

fn set_strategy_mode(
    goal: AxialHex,
    movement_budget: u32,
    range_radius: u32,
    attack_range: bool,
    reroute_barrier_enabled: bool,
) -> Action {
    set_control(move |control| {
        control.show_attack_range = attack_range;
        control.movement_budget = movement_budget;
        control.range_radius = range_radius;
        control.path_goal = goal;
        control.reroute_barrier_enabled = reroute_barrier_enabled;
    })
}

fn set_attack_range_visible(visible: bool) -> Action {
    set_control(move |control| {
        control.show_attack_range = visible;
    })
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

fn wait_for_lab_ready(label: impl Into<String>) -> Action {
    wait_for_diagnostics(label, |diagnostics| {
        diagnostics.path_exists
            && diagnostics.flat_hover_hex != diagnostics.pointy_hover_hex
            && diagnostics.reachable_count == 56
            && diagnostics.ring_count == 18
            && diagnostics.spiral_count == 37
            && diagnostics.fov_visible_count > 0
    })
}

fn build_smoke(name: &'static str) -> Scenario {
    Scenario::builder(name)
        .description("Boot the hex_grid lab, verify the layout, pathfinding, range, and FOV diagnostics all initialize, and capture the default showcase.")
        .then(wait_for_lab_ready("default diagnostics ready"))
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
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "fov overlay populated",
            |diagnostics| diagnostics.fov_visible_count > 0,
        ))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary(name))
        .build()
}

fn expected_range_fov_visible_count() -> usize {
    range_fov(AxialHex::ZERO, 4, |hex| crate::fov_wall_cells().contains(&hex))
        .into_iter()
        .filter(|hex| AxialHex::ZERO.distance_to(*hex) <= 5)
        .count()
}

fn expected_directional_fov_visible_count(direction: HexDirection) -> usize {
    directional_fov(AxialHex::ZERO, 4, direction, |hex| {
        crate::fov_wall_cells().contains(&hex)
    })
    .into_iter()
    .filter(|hex| AxialHex::ZERO.distance_to(*hex) <= 5)
    .count()
}

fn hex_grid_basic() -> Scenario {
    Scenario::builder("hex_grid_basic")
        .description("Move the shared sample point across the flat board, verify the hovered hex and its six neighbors update together, and capture the basic interaction state.")
        .then(wait_for_lab_ready("lab ready for basic interaction"))
        .then(set_sample_point(Vec2::new(34.0, -8.0)))
        .then(wait_for_diagnostics("basic hover ready", |diagnostics| {
            diagnostics.flat_hover_hex == AxialHex::new(1, -1)
                && diagnostics.flat_neighbor_count == 6
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "flat board hover coordinate",
            |diagnostics| diagnostics.flat_hover_hex == AxialHex::new(1, -1),
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "flat board highlights all six neighbors",
            |diagnostics| diagnostics.flat_neighbor_count == 6,
        ))
        .then(Action::Screenshot("basic_hover".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("hex_grid_basic"))
        .build()
}

fn hex_grid_layouts() -> Scenario {
    Scenario::builder("hex_grid_layouts")
        .description("Capture both the shared-origin case and an offset case to prove flat-top and pointy-top layouts map the same local point differently.")
        .then(wait_for_lab_ready("lab ready for layout comparison"))
        .then(set_sample_point(Vec2::ZERO))
        .then(wait_for_diagnostics("shared origin mapping", |diagnostics| {
            diagnostics.flat_hover_hex == AxialHex::ZERO
                && diagnostics.pointy_hover_hex == AxialHex::ZERO
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "shared origin maps flat board to origin",
            |diagnostics| diagnostics.flat_hover_hex == AxialHex::ZERO,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "shared origin maps pointy board to origin",
            |diagnostics| diagnostics.pointy_hover_hex == AxialHex::ZERO,
        ))
        .then(Action::Screenshot("layouts_origin".into()))
        .then(Action::WaitFrames(1))
        .then(set_sample_point(Vec2::new(36.0, -12.0)))
        .then(wait_for_diagnostics("offset mapping", |diagnostics| {
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
        .then(wait_for_lab_ready("lab ready for pathfinding"))
        .then(set_path_goal(AxialHex::new(3, -2), false))
        .then(wait_for_diagnostics("default path", |diagnostics| {
            diagnostics.path_exists && diagnostics.path_len == 6 && diagnostics.path_cost == 5
        }))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("path_default".into()))
        .then(Action::WaitFrames(1))
        .then(set_path_goal(AxialHex::new(4, -2), true))
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
        .then(wait_for_lab_ready("lab ready for range checks"))
        .then(set_range_mode(2, 2))
        .then(wait_for_diagnostics("tight range state", |diagnostics| {
            diagnostics.reachable_count == 15
                && diagnostics.ring_count == 12
                && diagnostics.spiral_count == 19
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "tight reachable count",
            |diagnostics| diagnostics.reachable_count == 15,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "tight ring count",
            |diagnostics| diagnostics.ring_count == 12,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "tight spiral count",
            |diagnostics| diagnostics.spiral_count == 19,
        ))
        .then(Action::Screenshot("ranges_tight".into()))
        .then(Action::WaitFrames(1))
        .then(set_range_mode(4, 3))
        .then(wait_for_diagnostics("expanded range state", |diagnostics| {
            diagnostics.ring_count == 18
                && diagnostics.spiral_count == 37
                && diagnostics.reachable_count == 56
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

fn hex_grid_fov() -> Scenario {
    Scenario::builder("hex_grid_fov")
        .description("Drive the lab's FOV board through both 360-degree and directional modes, capture each state, and verify the rendered visible set matches the algorithmic expectation.")
        .then(wait_for_lab_ready("lab ready for fov"))
        .then(set_fov_mode(AxialHex::ZERO, 4, false, HexDirection::East))
        .then(wait_for_diagnostics("range fov ready", |diagnostics| {
            diagnostics.fov_visible_count == expected_range_fov_visible_count()
        }))
        .then(assertions::custom("360 fov count matches board-visible algorithm", |world| {
            let diagnostics = world.resource::<crate::LabDiagnostics>();
            let expected = expected_range_fov_visible_count();
            diagnostics.fov_visible_count == expected && diagnostics.fov_hidden_behind_wall
        }))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("fov_range".into()))
        .then(Action::WaitFrames(1))
        .then(set_fov_mode(
            AxialHex::ZERO,
            4,
            true,
            HexDirection::East,
        ))
        .then(wait_for_diagnostics("directional fov ready", |diagnostics| {
            diagnostics.fov_visible_count == expected_directional_fov_visible_count(HexDirection::East)
        }))
        .then(assertions::custom("directional fov narrows the visible set", |world| {
            let diagnostics = world.resource::<crate::LabDiagnostics>();
            let range_expected = expected_range_fov_visible_count();
            let directional_expected = expected_directional_fov_visible_count(HexDirection::East);
            diagnostics.fov_visible_count == directional_expected
                && directional_expected < range_expected
        }))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("fov_directional".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("hex_grid_fov"))
        .build()
}

fn hex_grid_strategy() -> Scenario {
    Scenario::builder("hex_grid_strategy")
        .description("Switch the range board into strategy mode, verify reachability and attack coverage render together with the weighted path preview, and capture the tactical showcase.")
        .then(wait_for_lab_ready("lab ready for strategy"))
        .then(set_strategy_mode(AxialHex::new(3, -2), 4, 3, true, false))
        .then(wait_for_diagnostics("strategy state ready", |diagnostics| {
            diagnostics.path_exists
                && diagnostics.path_cost == 5
                && diagnostics.reachable_count == 56
                && diagnostics.attack_count > 0
        }))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "strategy preview path exists",
            |diagnostics| diagnostics.path_exists && diagnostics.path_cost == 5,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "strategy move range populated",
            |diagnostics| diagnostics.reachable_count == 56,
        ))
        .then(assertions::resource_satisfies::<crate::LabDiagnostics>(
            "strategy attack range populated",
            |diagnostics| diagnostics.attack_count > 0,
        ))
        .then(Action::Screenshot("strategy".into()))
        .then(Action::WaitFrames(1))
        .then(set_attack_range_visible(false))
        .then(assertions::log_summary("hex_grid_strategy"))
        .build()
}

fn hex_grid_storage() -> Scenario {
    Scenario::builder("hex_grid_storage")
        .description("Exercise HexagonalMap storage: create a map, read and mutate values, verify out-of-bounds returns None, and confirm iteration covers all hexes.")
        .then(wait_for_lab_ready("lab ready for storage screenshot"))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|_world: &mut World| {
            let center = AxialHex::ZERO;
            let radius = 2_u32;

            // Build a map with sequential initial values (index-based).
            let mut counter = 0_u32;
            let mut map: HexagonalMap<u32> = HexagonalMap::new(center, radius, |_hex| {
                let v = counter;
                counter += 1;
                v
            });

            // A radius-2 map has 1 + 6 + 12 = 19 hexes.
            assert_eq!(
                map.len(),
                19,
                "radius-2 HexagonalMap should have 19 hexes, got {}",
                map.len()
            );

            // get() on the center should return the value written at initialisation.
            let initial_center = *map.get(center).expect("center must be present");

            // Mutate the center value and verify the change.
            if let Some(v) = map.get_mut(center) {
                *v = 9999;
            }
            assert_eq!(
                map.get(center),
                Some(&9999),
                "mutated center value should be 9999"
            );
            assert_ne!(
                map.get(center),
                Some(&initial_center),
                "centre value must have changed after mutation"
            );

            // A hex outside the map radius should return None.
            let outside = AxialHex::new(10, 0);
            assert!(map.get(outside).is_none(), "out-of-bounds hex should return None");

            // Iteration must cover exactly 19 hexes and include the center.
            let mut found_center = false;
            let mut iter_count = 0_usize;
            for (hex, _val) in map.iter() {
                iter_count += 1;
                if hex == center {
                    found_center = true;
                }
            }
            assert_eq!(iter_count, 19, "iter should yield 19 hexes");
            assert!(found_center, "iter should include the center hex");

            for (hex, value) in map.iter_mut() {
                *value = hex.q.unsigned_abs() + hex.r.unsigned_abs();
            }
            assert_eq!(
                map.get(AxialHex::new(1, -1)),
                Some(&(1_u32 + 1_u32)),
                "iter_mut should update values in place"
            );
        })))
        .then(Action::Screenshot("hex_grid_storage".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("hex_grid_storage"))
        .build()
}
