use bevy::core_pipeline::core_3d::Camera3dBundle;
use bevy::ecs::prelude::ResMut;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Query;
use bevy::math::Vec3;
use bevy::render::camera::Camera;
use bevy::render::camera::RenderTarget;
use bevy::transform::components::Transform;
use bevy::window::PresentMode;
use bevy::window::PrimaryWindow;
use bevy::window::Window;
use bevy::window::WindowRef;
use bevy::window::WindowResolution;
use bevy::DefaultPlugins;
use bevy_egui::{EguiContext, EguiPlugin};
use egui::Align;
use egui::Label;
use egui::Layout;
use root_path::RootPath;

use sil::lex;

fn main() -> Result<(), &'static str> {
    let input = "[title]\nFoobar\n\n[text]\ncolor = 123";
    let tokens = lex(&input);
    println!("{:?}", tokens);

    bevy::prelude::App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(app::App::from(
            fetch_root_folder().ok_or("invalid root path")?,
        ))
        .add_startup_system(create_display_window)
        .add_systems((control_window, display_window))
        .run();

    Ok(())
}

fn fetch_root_folder() -> Option<RootPath> {
    let args: Vec<String> = std::env::args().collect();
    let path = if args.len() > 1 { &args[1] } else { "." };
    let file_path = std::path::Path::new(path);
    if !file_path.exists() || !file_path.is_dir() {
        return None
    }
    return Some(RootPath::from(path));
}

fn create_display_window(mut commands: Commands) {
    let display_window_id = commands
        .spawn(Window {
            title: "output".to_owned(),
            resolution: WindowResolution::new(800.0, 600.0),
            present_mode: PresentMode::AutoVsync,
            ..Default::default()
        })
        .id();
    commands.spawn(Camera3dBundle {
        camera: Camera {
            target: RenderTarget::Window(WindowRef::Entity(display_window_id)),
            ..Default::default()
        },
        transform: Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn control_window(
    mut app: ResMut<app::App>,
    mut egui_ctx: Query<&mut EguiContext, With<PrimaryWindow>>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else { return; };
    app.as_mut().update(ctx.get_mut());
}

fn display_window(
    app: ResMut<app::App>,
    mut egui_ctx: Query<&mut EguiContext, Without<PrimaryWindow>>,
) {
    let Ok(mut ctx) = egui_ctx.get_single_mut() else { return; };
    egui::CentralPanel::default().show(ctx.get_mut(), |ui| {
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            ui.add_sized(
                ui.available_size(),
                Label::new(app.as_ref().output().content.as_str()),
            );
        });
    });
}
