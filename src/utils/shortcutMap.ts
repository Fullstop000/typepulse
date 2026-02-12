const shortcutActionMap: Record<string, string> = {
  cmd_c: "Copy",
  cmd_v: "Paste",
  cmd_x: "Cut",
  cmd_s: "Save",
  cmd_w: "Close Window",
  cmd_z: "Undo",
  cmd_shift_z: "Redo",
  cmd_a: "Select All",
  cmd_f: "Find",
  cmd_b: "Build",
  cmd_p: "Command Palette",
  cmd_tab: "App Switcher",
  ctrl_c: "Copy",
  ctrl_v: "Paste",
  ctrl_x: "Cut",
  ctrl_s: "Save",
  ctrl_z: "Undo",
};

const tokenLabelMap: Record<string, string> = {
  ctrl: "Ctrl",
  opt: "Opt",
  shift: "Shift",
  cmd: "Cmd",
  space: "Space",
  enter: "Enter",
  tab: "Tab",
  esc: "Esc",
  backspace: "Backspace",
  delete: "Delete",
  left: "Left",
  right: "Right",
  up: "Up",
  down: "Down",
};

const legacyKeycodeMap: Record<number, string> = {
  24: "=",
  27: "-",
  33: "[",
  30: "]",
  41: ";",
  39: "'",
  42: "\\",
  43: ",",
  47: ".",
  44: "/",
};

// Best-effort label mapping for historical fallback tokens like `k27` / `K27`.
function legacyKeycodeTokenLabel(rawToken: string): string | null {
  const token = rawToken.trim().toLowerCase();
  const matched = /^k(\d+)$/.exec(token);
  if (!matched) {
    return null;
  }
  const code = Number(matched[1]);
  if (!Number.isFinite(code)) {
    return null;
  }
  return legacyKeycodeMap[code] ?? null;
}

// Convert canonical shortcut id to human-readable label.
export function formatShortcutLabel(shortcutId: string): string {
  return shortcutId
    .split("_")
    .map((rawToken) => {
      const token = rawToken.trim().toLowerCase();
      if (tokenLabelMap[token]) {
        return tokenLabelMap[token];
      }
      const legacyLabel = legacyKeycodeTokenLabel(token);
      if (legacyLabel) {
        return legacyLabel;
      }
      if (token.length === 1) {
        return token.toUpperCase();
      }
      return token;
    })
    .join(" + ");
}

// Lookup friendly shortcut action text from local mapping table.
export function shortcutActionLabel(shortcutId: string): string {
  return shortcutActionMap[shortcutId.toLowerCase()] ?? "Custom Action";
}
