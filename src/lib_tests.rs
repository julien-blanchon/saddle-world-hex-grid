use super::*;
use bevy::{app::App, gizmos::gizmos::GizmoStorage, prelude::MinimalPlugins};

#[test]
fn plugin_default_uses_update_schedule_and_stays_constructible() {
    let _plugin = HexGridPlugin::default();
}

#[test]
fn plugin_initializes_the_debug_gizmo_group() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(HexGridPlugin::default());

    assert!(
        app.world()
            .contains_resource::<GizmoStorage<HexGridDebugGizmos, ()>>()
    );
}
