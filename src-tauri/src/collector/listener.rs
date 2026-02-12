//! Platform listener module.
//! Converts OS keyboard callbacks into normalized collector events.

use std::sync::{Arc, Mutex};

use super::modifier::ModifierSnapshot;
#[cfg(not(target_os = "macos"))]
use super::modifier::ModifierState;
use super::{on_non_modifier_key_down, on_non_modifier_key_up, CollectorState};

#[cfg(not(target_os = "macos"))]
fn normalize_non_macos_key(key: rdev::Key) -> Option<String> {
    use rdev::Key;
    let normalized = match key {
        Key::KeyA => "a",
        Key::KeyB => "b",
        Key::KeyC => "c",
        Key::KeyD => "d",
        Key::KeyE => "e",
        Key::KeyF => "f",
        Key::KeyG => "g",
        Key::KeyH => "h",
        Key::KeyI => "i",
        Key::KeyJ => "j",
        Key::KeyK => "k",
        Key::KeyL => "l",
        Key::KeyM => "m",
        Key::KeyN => "n",
        Key::KeyO => "o",
        Key::KeyP => "p",
        Key::KeyQ => "q",
        Key::KeyR => "r",
        Key::KeyS => "s",
        Key::KeyT => "t",
        Key::KeyU => "u",
        Key::KeyV => "v",
        Key::KeyW => "w",
        Key::KeyX => "x",
        Key::KeyY => "y",
        Key::KeyZ => "z",
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",
        Key::Space => "space",
        Key::Return => "enter",
        Key::Tab => "tab",
        Key::Escape => "esc",
        Key::Backspace => "backspace",
        Key::Delete => "delete",
        Key::UpArrow => "up",
        Key::DownArrow => "down",
        Key::LeftArrow => "left",
        Key::RightArrow => "right",
        _ => return None,
    };
    Some(normalized.to_string())
}

#[cfg(not(target_os = "macos"))]
pub(super) fn on_key_event_non_macos(
    state: &Arc<Mutex<CollectorState>>,
    key: rdev::Key,
    pressed: bool,
) {
    let (is_modifier_key, modifiers_before) = if let Ok(mut locked) = state.lock() {
        let is_modifier_key = ModifierState::is_modifier_key(key);
        let modifiers_before = locked.modifier_state.snapshot();
        locked.modifier_state.update(key, pressed);
        (is_modifier_key, modifiers_before)
    } else {
        return;
    };

    if is_modifier_key {
        return;
    }

    let shortcut_key =
        normalize_non_macos_key(key).unwrap_or_else(|| format!("{:?}", key).to_lowercase());
    let physical_key_id = format!("rdev:{:?}", key);
    if pressed {
        on_non_modifier_key_down(
            state,
            physical_key_id,
            shortcut_key,
            modifiers_before,
            modifiers_before.has_any(),
        );
    } else {
        on_non_modifier_key_up(state, &physical_key_id, &shortcut_key, modifiers_before);
    }
}

#[cfg(target_os = "macos")]
pub(super) fn listen_keypress_macos(state: Arc<Mutex<CollectorState>>) -> Result<(), String> {
    use std::ffi::c_void;

    type CFMachPortRef = *const c_void;
    type CFIndex = i64;
    type CFAllocatorRef = *const c_void;
    type CFRunLoopSourceRef = *const c_void;
    type CFRunLoopRef = *const c_void;
    type CFRunLoopMode = *const c_void;

    type CGEventTapProxy = *const c_void;
    type CGEventRef = *const c_void;
    type CGEventTapLocation = u32;
    type CGEventTapPlacement = u32;
    type CGEventTapOptions = u32;
    type CGEventMask = u64;
    type CGEventType = u32;

    const CG_EVENT_TAP_LOCATION_HID: CGEventTapLocation = 0;
    const CG_EVENT_TAP_PLACEMENT_HEAD_INSERT: CGEventTapPlacement = 0;
    const CG_EVENT_TAP_OPTION_LISTEN_ONLY: CGEventTapOptions = 1;
    const CG_EVENT_TYPE_KEY_DOWN: CGEventType = 10;
    const CG_EVENT_TYPE_KEY_UP: CGEventType = 11;
    type CGEventFlags = u64;
    const CG_EVENT_FLAG_MASK_SHIFT: CGEventFlags = 1 << 17;
    const CG_EVENT_FLAG_MASK_CONTROL: CGEventFlags = 1 << 18;
    const CG_EVENT_FLAG_MASK_ALTERNATE: CGEventFlags = 1 << 19;
    const CG_EVENT_FLAG_MASK_COMMAND: CGEventFlags = 1 << 20;
    const CG_EVENT_FLAG_MASK_SECONDARY_FN: CGEventFlags = 1 << 23;
    type CGEventField = u32;
    const CG_EVENT_FIELD_KEYBOARD_EVENT_KEYCODE: CGEventField = 9;

    fn snapshot_from_macos_flags(flags: CGEventFlags) -> ModifierSnapshot {
        ModifierSnapshot {
            ctrl: flags & CG_EVENT_FLAG_MASK_CONTROL != 0,
            opt: flags & CG_EVENT_FLAG_MASK_ALTERNATE != 0,
            shift: flags & CG_EVENT_FLAG_MASK_SHIFT != 0,
            cmd: flags & CG_EVENT_FLAG_MASK_COMMAND != 0,
            function: flags & CG_EVENT_FLAG_MASK_SECONDARY_FN != 0,
        }
    }

    fn normalize_macos_keycode(key_code: i64) -> String {
        let key = match key_code {
            0 => "a",
            1 => "s",
            2 => "d",
            3 => "f",
            4 => "h",
            5 => "g",
            6 => "z",
            7 => "x",
            8 => "c",
            9 => "v",
            11 => "b",
            12 => "q",
            13 => "w",
            14 => "e",
            15 => "r",
            16 => "y",
            17 => "t",
            31 => "o",
            32 => "u",
            34 => "i",
            35 => "p",
            37 => "l",
            38 => "j",
            40 => "k",
            45 => "n",
            46 => "m",
            18 => "1",
            19 => "2",
            20 => "3",
            21 => "4",
            23 => "5",
            22 => "6",
            26 => "7",
            28 => "8",
            25 => "9",
            29 => "0",
            24 => "=",
            27 => "-",
            33 => "[",
            30 => "]",
            41 => ";",
            39 => "'",
            42 => "\\",
            43 => ",",
            47 => ".",
            44 => "/",
            36 => "enter",
            48 => "tab",
            49 => "space",
            51 => "backspace",
            53 => "esc",
            123 => "left",
            124 => "right",
            125 => "down",
            126 => "up",
            _ => return format!("k{key_code}"),
        };
        key.to_string()
    }

    extern "C" {
        fn CGEventTapCreate(
            tap: CGEventTapLocation,
            place: CGEventTapPlacement,
            options: CGEventTapOptions,
            events_of_interest: CGEventMask,
            callback: unsafe extern "C" fn(
                CGEventTapProxy,
                CGEventType,
                CGEventRef,
                *mut c_void,
            ) -> CGEventRef,
            user_info: *mut c_void,
        ) -> CFMachPortRef;
        fn CFMachPortCreateRunLoopSource(
            allocator: CFAllocatorRef,
            port: CFMachPortRef,
            order: CFIndex,
        ) -> CFRunLoopSourceRef;
        fn CFRunLoopGetCurrent() -> CFRunLoopRef;
        fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
        fn CFRunLoopRun();
        fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        fn CGEventGetFlags(event: CGEventRef) -> CGEventFlags;
        fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
        static kCFRunLoopCommonModes: CFRunLoopMode;
    }

    unsafe extern "C" fn callback(
        _proxy: CGEventTapProxy,
        type_: CGEventType,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef {
        if type_ == CG_EVENT_TYPE_KEY_DOWN || type_ == CG_EVENT_TYPE_KEY_UP {
            let state = &*(user_info as *const Arc<Mutex<CollectorState>>);
            let flags = CGEventGetFlags(event);
            let key_code =
                CGEventGetIntegerValueField(event, CG_EVENT_FIELD_KEYBOARD_EVENT_KEYCODE);
            let physical_key_id = format!("mac:{key_code}");
            let shortcut_key = normalize_macos_keycode(key_code);
            let modifiers = snapshot_from_macos_flags(flags);
            if type_ == CG_EVENT_TYPE_KEY_DOWN {
                on_non_modifier_key_down(
                    state,
                    physical_key_id,
                    shortcut_key,
                    modifiers,
                    modifiers.has_any(),
                );
            } else {
                on_non_modifier_key_up(state, &physical_key_id, &shortcut_key, modifiers);
            }
        }
        event
    }

    let user_info = Box::into_raw(Box::new(state)) as *mut c_void;
    unsafe {
        let tap = CGEventTapCreate(
            CG_EVENT_TAP_LOCATION_HID,
            CG_EVENT_TAP_PLACEMENT_HEAD_INSERT,
            CG_EVENT_TAP_OPTION_LISTEN_ONLY,
            (1u64 << CG_EVENT_TYPE_KEY_DOWN) | (1u64 << CG_EVENT_TYPE_KEY_UP),
            callback,
            user_info,
        );
        if tap.is_null() {
            return Err("EventTapCreate failed (need Accessibility permission?)".to_string());
        }
        let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
        if source.is_null() {
            return Err("CFMachPortCreateRunLoopSource failed".to_string());
        }
        let run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
    Ok(())
}
