//! Modifier key model module.
//! Defines normalized modifier snapshots and non-mac modifier runtime state.

/// Modifier snapshot used by shortcut normalization and event serialization.
#[derive(Clone, Copy, Default)]
pub(super) struct ModifierSnapshot {
    pub(super) ctrl: bool,
    pub(super) opt: bool,
    pub(super) shift: bool,
    pub(super) cmd: bool,
    pub(super) function: bool,
}

impl ModifierSnapshot {
    pub(super) fn has_any(&self) -> bool {
        self.ctrl || self.opt || self.shift || self.cmd || self.function
    }

    pub(super) fn has_shortcut_modifier(&self) -> bool {
        self.ctrl || self.cmd
    }

    pub(super) fn bitmask(&self) -> u8 {
        (self.ctrl as u8)
            | ((self.opt as u8) << 1)
            | ((self.shift as u8) << 2)
            | ((self.cmd as u8) << 3)
            | ((self.function as u8) << 4)
    }

    // Rebuild modifier snapshot from persisted bitmask in compact input events.
    pub(super) fn from_bitmask(mask: u8) -> Self {
        Self {
            ctrl: (mask & 0b00001) != 0,
            opt: (mask & 0b00010) != 0,
            shift: (mask & 0b00100) != 0,
            cmd: (mask & 0b01000) != 0,
            function: (mask & 0b10000) != 0,
        }
    }

    pub(super) fn modifier_count(&self) -> u8 {
        self.ctrl as u8 + self.opt as u8 + self.shift as u8 + self.cmd as u8 + self.function as u8
    }
}

#[cfg(not(target_os = "macos"))]
#[derive(Default)]
pub(super) struct ModifierState {
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
    function: bool,
}

#[cfg(not(target_os = "macos"))]
impl ModifierState {
    pub(super) fn has_any_modifier(&self) -> bool {
        self.ctrl || self.alt || self.shift || self.meta || self.function
    }

    pub(super) fn is_modifier_key(key: rdev::Key) -> bool {
        matches!(
            key,
            rdev::Key::ControlLeft
                | rdev::Key::ControlRight
                | rdev::Key::Alt
                | rdev::Key::AltGr
                | rdev::Key::ShiftLeft
                | rdev::Key::ShiftRight
                | rdev::Key::MetaLeft
                | rdev::Key::MetaRight
                | rdev::Key::Function
        )
    }

    pub(super) fn update(&mut self, key: rdev::Key, pressed: bool) {
        match key {
            rdev::Key::ControlLeft | rdev::Key::ControlRight => self.ctrl = pressed,
            rdev::Key::Alt | rdev::Key::AltGr => self.alt = pressed,
            rdev::Key::ShiftLeft | rdev::Key::ShiftRight => self.shift = pressed,
            rdev::Key::MetaLeft | rdev::Key::MetaRight => self.meta = pressed,
            rdev::Key::Function => self.function = pressed,
            _ => {}
        }
    }

    pub(super) fn snapshot(&self) -> ModifierSnapshot {
        ModifierSnapshot {
            ctrl: self.ctrl,
            opt: self.alt,
            shift: self.shift,
            cmd: self.meta,
            function: self.function,
        }
    }
}
