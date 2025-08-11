use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use chrono::{DateTime, Local, TimeZone};

// Target broadcast time (local time)
const TARGET_TIME_STR: &str = "2025-12-31T23:59:59"; // yyyy-mm-ddTHH:MM:SS

// Offscreen Mandelbrot render size
const MANDEL_WIDTH: u32 = 640;
const MANDEL_HEIGHT: u32 = 360;

#[derive(Resource)]
struct TargetTime(DateTime<Local>);

#[derive(Resource)]
struct MandelState {
    image: Handle<Image>,
    // Zoom handling: smaller => closer (exponential over a cycle)
    zoom: f32,
    center: Vec2,   // complex plane center
    fps_timer: Timer,
    // New: 90s zoom cycles across interesting targets
    start_zoom: f32,
    min_zoom: f32,
    cycle_timer: Timer,
    targets: Vec<Vec2>,
    target_index: usize,
}

#[derive(Component)]
struct MandelSprite;

#[derive(Component)]
struct CountdownText;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Standby - Mandelbrot + Countdown".to_string(),
                    resolution: (1280.0, 720.0).into(),
                    resizable: true,
                    ..default()
                }),
                ..default()
            }),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, (setup_camera, setup_mandelbrot, setup_text))
        .add_systems(Update, (update_mandelbrot, fit_sprite_to_window, update_countdown_text))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_mandelbrot(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
) {
    let size = Extent3d {
        width: MANDEL_WIDTH,
        height: MANDEL_HEIGHT,
        depth_or_array_layers: 1,
    };
    let pixel_count = (MANDEL_WIDTH * MANDEL_HEIGHT) as usize;
    let data = vec![0u8; pixel_count * 4];
    let image = Image::new_fill(
        size,
        TextureDimension::D2,
        &data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );

    let image_handle = images.add(image);

    let window = windows.single().expect("primary window");
    let w = window.width();
    let h = window.height();

    commands
        .spawn((
            Sprite {
                image: image_handle.clone(),
                custom_size: Some(Vec2::new(w, h)),
                ..default()
            },
            Transform::default(),
            Visibility::Visible,
            MandelSprite,
        ));

    // Curated set of visually rich target centers near the boundary
    let targets = vec![
        Vec2::new(-0.743_643_887_037_151, 0.131_825_904_205_33), // Seahorse valley
        Vec2::new(-1.250_66, 0.020_12),                           // Antenna region
        Vec2::new(0.001_643_721_971_153, 0.822_467_633_298_876), // Spiral filaments
        Vec2::new(-0.101_096_363_845, 0.956_286_510_809),        // Boundary filigree
        Vec2::new(-0.8, 0.156),                                   // Mini-set chain
    ];

    commands.insert_resource(MandelState {
        image: image_handle,
        start_zoom: 3.0,
        min_zoom: 0.000_6, // stop before precision/interior dominates
        zoom: 3.0,
        center: targets[0],
        fps_timer: Timer::from_seconds(1.0 / 20.0, TimerMode::Repeating), // ~20 FPS updates
        cycle_timer: Timer::from_seconds(90.0, TimerMode::Repeating),     // 90s continuous zoom cycles
        targets,
        target_index: 0,
    });

    let parsed_local = Local
        .datetime_from_str(TARGET_TIME_STR, "%Y-%m-%dT%H:%M:%S")
        .unwrap_or_else(|_| Local::now());
    commands.insert_resource(TargetTime(parsed_local));
}

fn mandelbrot_color(iter: u32, max_iter: u32) -> [u8; 4] {
    // Keep the interior perfectly black; avoid any reddish palette for exterior.
    if iter >= max_iter {
        return [0, 0, 0, 0xFF];
    }

    let t = iter as f32 / max_iter as f32;
    // Dark, space-like palette biased toward cyan/blue (no red component)
    let intensity = t.powf(0.35);
    let g = (intensity * 180.0) as u8;
    let b = (intensity * 255.0) as u8;
    [0, g, b, 0xFF]
}

fn update_mandelbrot(
    time: Res<Time>,
    mut images: ResMut<Assets<Image>>,
    mut state: ResMut<MandelState>,
    windows: Query<&Window>,
) {
    if !state.fps_timer.tick(time.delta()).just_finished() {
        return;
    }

    // Drive 90s zoom cycle and advance target when a cycle completes
    let cycle_finished = state.cycle_timer.tick(time.delta()).just_finished();
    if cycle_finished {
        state.target_index = (state.target_index + 1) % state.targets.len();
    }

    // Smooth exponential zoom: start_zoom -> min_zoom over the cycle duration
    let cycle_elapsed = state.cycle_timer.elapsed().as_secs_f32();
    let cycle_total = state.cycle_timer.duration().as_secs_f32();
    let progress = (cycle_elapsed / cycle_total).clamp(0.0, 1.0);
    state.zoom = state.start_zoom * (state.min_zoom / state.start_zoom).powf(progress);

    // Keep near boundary with a subtle orbit around the current target
    let base = state.targets[state.target_index];
    let t = time.elapsed_secs();
    let orbit = Vec2::new((t * 0.11).sin(), (t * 0.13).cos()) * (0.35 * state.zoom);
    state.center = base + orbit;

    let Some(image) = images.get_mut(&state.image) else { return; };

    let width = image.texture_descriptor.size.width as i32;
    let height = image.texture_descriptor.size.height as i32;

    let window = windows.single().expect("primary window");
    let aspect = window.width() / window.height();

    // Increase iterations as we zoom in to retain edge detail
    let zoom_factor = (state.start_zoom / state.zoom).max(1.0);
    let dynamic_iters = 80.0 + 20.0 * zoom_factor.log2();
    let max_iter: u32 = dynamic_iters.clamp(80.0, 1024.0) as u32;

    let Some(ref mut data) = image.data else { return; };
    let mut interior_pixels: u32 = 0;

    for y in 0..height {
        for x in 0..width {
            let u = (x as f32 + 0.5) / width as f32;
            let v = (y as f32 + 0.5) / height as f32;

            let mut cx = (u - 0.5) * 2.0 * aspect;
            let mut cy = (v - 0.5) * 2.0;
            cx = cx * state.zoom + state.center.x;
            cy = cy * state.zoom + state.center.y;

            let mut zx = 0.0f32;
            let mut zy = 0.0f32;
            let mut it = 0u32;
            while it < max_iter {
                let x2 = zx * zx - zy * zy + cx;
                let y2 = 2.0 * zx * zy + cy;
                zx = x2;
                zy = y2;
                if zx * zx + zy * zy > 4.0 {
                    break;
                }
                it += 1;
            }

            let color = mandelbrot_color(it, max_iter);
            let idx = ((y as u32 * MANDEL_WIDTH + x as u32) * 4) as usize;
            data[idx..idx + 4].copy_from_slice(&color);

            if it >= max_iter {
                interior_pixels += 1;
            }
        }
    }

    // If nearly all pixels are interior (black), advance cycle early to avoid a black-only screen
    let total_pixels = (width as u32 * height as u32) as u32;
    if interior_pixels as f32 / total_pixels as f32 > 0.985 {
        state.target_index = (state.target_index + 1) % state.targets.len();
        state.cycle_timer.reset();
        state.zoom = state.start_zoom;
        state.center = state.targets[state.target_index];
    }
}

fn fit_sprite_to_window(
    windows: Query<&Window>,
    mut sprites: Query<(&mut Sprite, &mut Transform), With<MandelSprite>>,
) {
    let window = windows.single().expect("primary window");
    if let Ok((mut sprite, mut transform)) = sprites.get_single_mut() {
        sprite.custom_size = Some(Vec2::new(window.width(), window.height()));
        transform.translation.z = 0.0;
    }
}

fn setup_text(mut commands: Commands) {
    commands.spawn((
        Text::new("--:--:--"),
        TextFont { font_size: 140.0, ..default() },
        TextLayout { justify: JustifyText::Center, ..default() },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        Visibility::Visible,
        CountdownText,
    ));
}

fn update_countdown_text(
    target: Res<TargetTime>,
    mut texts: Query<(&mut Text, &mut TextColor, &mut TextFont), With<CountdownText>>,
) {
    let now = Local::now();
    let remaining = target.0 - now;
    let total_secs = remaining.num_seconds();

    let (text_value, color) = if total_secs >= 0 {
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;
        (format!("{:02}:{:02}:{:02}", hours, minutes, seconds), Color::WHITE)
    } else {
        ("LIVE".to_string(), Color::srgb_u8(0, 255, 128))
    };

    if let Ok((mut text, mut text_color, mut font)) = texts.get_single_mut() {
        *text = Text::new(text_value);
        *text_color = TextColor(color);
        font.font_size = 140.0;
    }
}
