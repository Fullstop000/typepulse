//! Capture context module.
//! Resolves frontmost app/window/security context and defines collector event payloads.

use std::path::PathBuf;

use serde::Serialize;

use super::modifier::ModifierSnapshot;
use super::CollectorState;
use std::time::{Duration, Instant};

/// Running app info payload for exclusion management UI.
#[derive(Serialize, Clone)]
pub struct RunningAppInfo {
    pub bundle_id: String,
    pub name: String,
}

#[derive(Clone)]
pub(super) struct CaptureContext {
    pub(super) app_name: String,
    pub(super) window_title: String,
    pub(super) bundle_id: Option<String>,
    pub(super) secure_input: bool,
}

// Unified collector event model used by runtime handlers and unit tests.
#[derive(Clone)]
pub(super) enum CollectorEvent {
    NonModifierKeyDown {
        physical_key_id: String,
        shortcut_key: String,
        modifiers: ModifierSnapshot,
        is_key_combo: bool,
        capture_context: CaptureContext,
        at: Instant,
    },
    NonModifierKeyUp {
        physical_key_id: String,
        shortcut_key: String,
        modifiers: ModifierSnapshot,
        capture_context: CaptureContext,
    },
    Tick {
        elapsed: Duration,
        capture_context: CaptureContext,
        at: Instant,
    },
}

pub(super) fn capture_context() -> CaptureContext {
    let secure_input = is_secure_event_input_enabled();
    if let Ok(window) = active_win_pos_rs::get_active_window() {
        let app_name = window.app_name;
        let window_title = window.title;
        #[cfg(target_os = "macos")]
        {
            let bundle_id = frontmost_bundle_id_macos()
                .or_else(|| bundle_id_from_path(&window.process_path))
                .map(|v| v.to_ascii_lowercase());
            return CaptureContext {
                app_name,
                window_title,
                bundle_id,
                secure_input,
            };
        }
        #[cfg(not(target_os = "macos"))]
        {
            return CaptureContext {
                app_name,
                window_title,
                bundle_id: None,
                secure_input,
            };
        }
    }
    CaptureContext {
        app_name: "Unknown".to_string(),
        window_title: String::new(),
        bundle_id: None,
        secure_input,
    }
}

pub(super) fn is_auto_paused(state: &CollectorState, context: &CaptureContext) -> bool {
    is_excluded_app(state, context) || context.secure_input
}

pub(super) fn auto_pause_reason(
    state: &CollectorState,
    context: &CaptureContext,
) -> Option<String> {
    if context.secure_input {
        return Some("secure_input".to_string());
    }
    if is_excluded_app(state, context) {
        return Some("blacklist".to_string());
    }
    None
}

fn is_excluded_app(state: &CollectorState, context: &CaptureContext) -> bool {
    match &context.bundle_id {
        Some(bundle_id) => state
            .excluded_bundle_ids
            .contains(&bundle_id.to_ascii_lowercase()),
        None => false,
    }
}

pub fn running_apps() -> Vec<RunningAppInfo> {
    #[cfg(target_os = "macos")]
    {
        return workspace_running_apps();
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec![]
    }
}

pub fn bundle_id_from_app_path(path: &str) -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        bundle_id_from_path(std::path::Path::new(path)).map(|v| v.to_ascii_lowercase())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        None
    }
}

#[cfg(target_os = "macos")]
fn bundle_id_from_path(path: &std::path::Path) -> Option<String> {
    let mut bundle_path: Option<PathBuf> = None;
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if let Some(name) = current.file_name().and_then(|v| v.to_str()) {
            if name.ends_with(".app") {
                bundle_path = Some(current.clone());
            }
        }
    }
    let bundle_path = bundle_path?;
    let info_plist = bundle_path.join("Contents").join("Info.plist");
    let value = plist::Value::from_file(info_plist).ok()?;
    match value.as_dictionary() {
        Some(dict) => dict
            .get("CFBundleIdentifier")
            .and_then(|v| v.as_string())
            .map(|v| v.to_string()),
        None => None,
    }
}

#[cfg(target_os = "macos")]
fn is_secure_event_input_enabled() -> bool {
    #[link(name = "Carbon", kind = "framework")]
    extern "C" {
        fn IsSecureEventInputEnabled() -> bool;
    }
    unsafe { IsSecureEventInputEnabled() }
}

#[cfg(not(target_os = "macos"))]
fn is_secure_event_input_enabled() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn frontmost_bundle_id_macos() -> Option<String> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return None;
        }
        let app: id = msg_send![workspace, frontmostApplication];
        if app == nil {
            return None;
        }
        let bundle_id: id = msg_send![app, bundleIdentifier];
        if bundle_id == nil {
            return None;
        }
        Some(nsstring_to_string(bundle_id))
    }
}

#[cfg(target_os = "macos")]
fn workspace_running_apps() -> Vec<RunningAppInfo> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use std::collections::BTreeMap;

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        if workspace == nil {
            return vec![];
        }
        let apps: id = msg_send![workspace, runningApplications];
        if apps == nil {
            return vec![];
        }
        let count: usize = msg_send![apps, count];
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for idx in 0..count {
            let app: id = msg_send![apps, objectAtIndex: idx];
            if app == nil {
                continue;
            }
            let bundle_id_obj: id = msg_send![app, bundleIdentifier];
            if bundle_id_obj == nil {
                continue;
            }
            let name_obj: id = msg_send![app, localizedName];
            let bundle_id = nsstring_to_string(bundle_id_obj);
            let name = if name_obj == nil {
                bundle_id.clone()
            } else {
                nsstring_to_string(name_obj)
            };
            if !bundle_id.trim().is_empty() {
                map.insert(bundle_id.to_ascii_lowercase(), name);
            }
        }
        map.into_iter()
            .map(|(bundle_id, name)| RunningAppInfo { bundle_id, name })
            .collect()
    }
}

#[cfg(target_os = "macos")]
fn nsstring_to_string(value: cocoa::base::id) -> String {
    use objc::{msg_send, sel, sel_impl};

    unsafe {
        let bytes: *const std::os::raw::c_char = msg_send![value, UTF8String];
        if bytes.is_null() {
            return String::new();
        }
        std::ffi::CStr::from_ptr(bytes)
            .to_string_lossy()
            .to_string()
    }
}
