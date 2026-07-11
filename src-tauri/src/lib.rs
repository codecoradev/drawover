pub mod drawing;

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
// global shortcut disabled during dev

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// A single drawing stroke rendered on the overlay.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Stroke {
    pub id: String,
    pub points: Vec<(f32, f32)>,
    pub color: String,
    pub width: f32,
    pub tool: String,
    pub opacity: f32,
}

/// Feature gate — during MVP all features are free.
/// `is_pro()` returns `true` when the user has NOT activated a pro license,
/// meaning every feature is unlocked (freemium MVP mode).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FeatureGate {
    pub is_pro_activated: bool,
}

impl FeatureGate {
    /// Returns `true` when pro features should be available.
    /// During MVP: pro is always available (is_pro_activated == false → is_pro() == true).
    pub fn is_pro(&self) -> bool {
        !self.is_pro_activated
    }
}

/// Central application state shared across Tauri commands.
#[derive(Default)]
pub struct AppState {
    pub strokes: Vec<Stroke>,
    pub undo_stack: Vec<Stroke>,
    pub redo_stack: Vec<Stroke>,
    pub current_stroke: Option<Stroke>,
    pub is_draw_mode: bool,
    pub feature_gate: FeatureGate,
}

// ---------------------------------------------------------------------------
// Window helpers
// ---------------------------------------------------------------------------

/// Toggle whether the overlay window ignores mouse events (click-through).
/// In Tauri 2, `set_ignore_cursor_events` is available on all platforms
/// (it's a no-op on platforms that don't support it).
fn apply_click_through(app: &tauri::AppHandle, ignore: bool) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.set_ignore_cursor_events(ignore);
    }
}

/// Show the overlay window and capture mouse events.
fn show_overlay(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        // Force window to cover the primary screen (fixes maximized/center conflict on macOS)
        if let Ok(monitor) = window.current_monitor() {
            if let Some(monitor) = monitor {
                let size = monitor.size();
                let pos = monitor.position();
                let _ = window.set_position(tauri::PhysicalPosition::new(pos.x, pos.y));
                let _ = window.set_size(tauri::PhysicalSize::new(size.width, size.height));
            }
        }
        let _ = window.show();
        let _ = window.set_focus();
        // Re-assert always-on-top so the overlay sits above every app
        let _ = window.set_always_on_top(true);
        let r = window.set_ignore_cursor_events(false);
        println!(
            "[DrawOver] show_overlay — focus+ignore_cursor result: {:?}",
            r
        );
    } else {
        eprintln!("[DrawOver] show_overlay — overlay window not found!");
    }
}

// ---------------------------------------------------------------------------
// Core draw-mode toggle logic (shared between command, tray, and shortcut)
// ---------------------------------------------------------------------------

fn do_toggle_draw_mode(state: &Mutex<AppState>, app: &tauri::AppHandle) -> bool {
    let mode = {
        let mut s = match state.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        s.is_draw_mode = !s.is_draw_mode;
        s.is_draw_mode
    };
    println!("[DrawOver] do_toggle_draw_mode -> {}", mode);

    if mode {
        // Become a regular app so the overlay can take focus & capture mouse
        #[cfg(target_os = "macos")]
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
        show_overlay(app);
    } else {
        apply_click_through(app, true);
        // Return to accessory (tray-only) so the overlay is click-through
        #[cfg(target_os = "macos")]
        let _ = app.set_activation_policy(tauri::ActivationPolicy::Accessory);
    }

    let _ = app.emit("draw-mode-toggled", mode);
    mode
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// Toggle draw mode on/off.
#[tauri::command]
fn toggle_draw_mode(
    state: tauri::State<'_, Mutex<AppState>>,
    app: tauri::AppHandle,
) -> Result<bool, String> {
    println!("[DrawOver] toggle_draw_mode COMMAND called");
    Ok(do_toggle_draw_mode(state.inner(), &app))
}

/// Begin a new stroke.
#[tauri::command]
fn start_stroke(
    state: tauri::State<'_, Mutex<AppState>>,
    color: String,
    width: f32,
    tool: String,
) -> Result<String, String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    // Check feature gate for pro tools
    let pro_tools = ["highlighter", "arrow", "rectangle", "ellipse"];
    if pro_tools.contains(&tool.as_str()) && !s.feature_gate.is_pro() {
        return Err("Pro feature not available".to_string());
    }

    let id = uuid::Uuid::new_v4().to_string();
    s.current_stroke = Some(Stroke {
        id: id.clone(),
        points: Vec::new(),
        color,
        width,
        tool,
        opacity: 1.0,
    });
    Ok(id)
}

/// Append a point to the current stroke.
#[tauri::command]
fn add_point(state: tauri::State<'_, Mutex<AppState>>, x: f32, y: f32) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(stroke) = &mut s.current_stroke {
        stroke.points.push((x, y));
    }
    Ok(())
}

/// Finalize the current stroke and push it to the strokes list.
#[tauri::command]
fn end_stroke(
    state: tauri::State<'_, Mutex<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(stroke) = s.current_stroke.take() {
        // Only store strokes that have at least one point
        if !stroke.points.is_empty() {
            s.strokes.push(stroke);
            s.redo_stack.clear(); // Clear redo on new action
            let _ = app.emit("strokes-updated", &s.strokes);
        }
    }
    Ok(())
}

/// Undo: pop last stroke to redo stack.
#[tauri::command]
fn undo(state: tauri::State<'_, Mutex<AppState>>, app: tauri::AppHandle) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(stroke) = s.strokes.pop() {
        s.redo_stack.push(stroke);
        let _ = app.emit("strokes-updated", &s.strokes);
    }
    Ok(())
}

/// Redo: pop from redo stack back to strokes.
#[tauri::command]
fn redo(state: tauri::State<'_, Mutex<AppState>>, app: tauri::AppHandle) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    if let Some(stroke) = s.redo_stack.pop() {
        s.strokes.push(stroke);
        let _ = app.emit("strokes-updated", &s.strokes);
    }
    Ok(())
}

/// Clear all strokes.
#[tauri::command]
fn clear_all(
    state: tauri::State<'_, Mutex<AppState>>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.strokes.clear();
    s.undo_stack.clear();
    s.redo_stack.clear();
    s.current_stroke = None;
    let _ = app.emit("strokes-updated", &s.strokes);
    Ok(())
}

/// Return all strokes as JSON-serializable data.
#[tauri::command]
fn get_strokes(state: tauri::State<'_, Mutex<AppState>>) -> Result<Vec<Stroke>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.strokes.clone())
}

/// Toggle click-through for the overlay window.
#[tauri::command]
fn set_ignore_mouse_events(ignore: bool, app: tauri::AppHandle) -> Result<(), String> {
    apply_click_through(&app, ignore);
    Ok(())
}

/// Check if draw mode is active.
#[tauri::command]
fn is_draw_mode(state: tauri::State<'_, Mutex<AppState>>) -> Result<bool, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.is_draw_mode)
}

// ---------------------------------------------------------------------------
// Tray icon setup
// ---------------------------------------------------------------------------

fn setup_tray(app: &tauri::AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let toggle_item = MenuItem::with_id(app, "toggle", "Toggle Draw Mode", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit DrawOver", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or("default window icon not found")?;

    TrayIconBuilder::new()
        .icon(icon)
        .tooltip("DrawOver")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "toggle" => {
                let state = app.state::<Mutex<AppState>>();
                do_toggle_draw_mode(state.inner(), app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--drawover"]),
        ))
        .manage(Mutex::new(AppState::default()))
        .setup(|app| {
            let handle = app.handle();

            // ----- Tray icon -----
            setup_tray(handle)?;

            // ----- Global shortcut: DISABLED during dev (phantom toggle issue) -----
            // Re-enable once focus/permission issues are resolved.
            // let shortcut: Shortcut = "Alt+Shift+D".parse()?;
            // println!("[DrawOver] registering global shortcut: {:?}", shortcut);
            // handle
            //     .global_shortcut()
            //     .on_shortcut(shortcut.clone(), move |app, _shortcut, event| {
            //         println!("[DrawOver] shortcut event: state={:?}", event.state);
            //         if event.state == ShortcutState::Pressed {
            //             let state = app.state::<Mutex<AppState>>();
            //             let new_mode = do_toggle_draw_mode(state.inner(), app);
            //             println!("[DrawOver] draw mode toggled -> {}", new_mode);
            //         }
            //     })
            //     .map_err(|e| {
            //         eprintln!("[DrawOver] FAILED to register global shortcut: {}", e);
            //         e
            //     })?;

            // ----- Initial state: overlay captures mouse so manual toggle works -----
            apply_click_through(app.handle(), false);
            println!("[DrawOver] startup complete — overlay clickable");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_draw_mode,
            start_stroke,
            add_point,
            end_stroke,
            undo,
            redo,
            clear_all,
            get_strokes,
            set_ignore_mouse_events,
            is_draw_mode,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DrawOver application");
}
