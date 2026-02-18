import { SystemStyleObject } from "@chakra-ui/react";

// Keep WebKit and standard backdrop filter aligned for glass surfaces.
function withWebkitBackdropFilter(value: string): Record<string, string> {
  return { WebkitBackdropFilter: value };
}

// Shared glass-like surface for top-level cards and panels.
export const glassSurfaceStyle: SystemStyleObject = {
  bg: "glass.bg",
  borderWidth: "1px",
  borderColor: "glass.border",
  backdropFilter: "blur(22px) saturate(1.15)",
  ...(withWebkitBackdropFilter("blur(22px) saturate(1.15)") as Record<string, string>),
  boxShadow: "glass",
};

// Sub-surface for nested blocks inside a glass container.
export const glassSubtleStyle: SystemStyleObject = {
  bg: "glass.subtle",
  borderWidth: "1px",
  borderColor: "glass.borderSoft",
  backdropFilter: "blur(16px) saturate(1.1)",
  ...(withWebkitBackdropFilter("blur(16px) saturate(1.1)") as Record<string, string>),
};

// Pill-style surface for segmented filters and quick actions.
export const glassPillStyle: SystemStyleObject = {
  bg: "glass.pill",
  borderWidth: "1px",
  borderColor: "glass.border",
  backdropFilter: "blur(14px) saturate(1.1)",
  ...(withWebkitBackdropFilter("blur(14px) saturate(1.1)") as Record<string, string>),
};

// Tray popover shell style to keep native-like glass density consistent.
export const trayPopoverSurfaceStyle: SystemStyleObject = {
  bg: "rgba(242, 246, 252, 0.14)",
  borderWidth: "1px",
  borderColor: "rgba(255,255,255,0.46)",
  backdropFilter: "blur(52px) saturate(1.65) brightness(1.04)",
  ...(withWebkitBackdropFilter("blur(52px) saturate(1.65) brightness(1.04)") as Record<
    string,
    string
  >),
  boxShadow: "0 10px 24px rgba(15,23,42,0.12), inset 0 1px 0 rgba(255,255,255,0.66)",
};

// Overlay highlight for tray popover shell.
export const trayPopoverOverlayStyle: SystemStyleObject = {
  bg: "linear-gradient(180deg, rgba(255,255,255,0.14) 0%, rgba(255,255,255,0.02) 48%, rgba(255,255,255,0.09) 100%)",
};

// Compact metric card style for tray stats blocks.
export const trayMetricCardStyle: SystemStyleObject = {
  bg: "rgba(255,255,255,0.12)",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  borderWidth: "1px",
  borderColor: "rgba(255,255,255,0.3)",
  backdropFilter: "blur(18px) saturate(1.2)",
  ...(withWebkitBackdropFilter("blur(18px) saturate(1.2)") as Record<string, string>),
  boxShadow: "0 3px 8px rgba(15,23,42,0.06), inset 0 1px 0 rgba(255,255,255,0.42)",
};

// Base action button style for tray controls.
export const trayActionButtonStyle: SystemStyleObject = {
  bg: "rgba(255,255,255,0.14)",
  borderWidth: "1px",
  borderColor: "rgba(255,255,255,0.36)",
  backdropFilter: "blur(18px) saturate(1.22)",
  ...(withWebkitBackdropFilter("blur(18px) saturate(1.22)") as Record<string, string>),
  boxShadow: "0 4px 10px rgba(15,23,42,0.08), inset 0 1px 0 rgba(255,255,255,0.48)",
  _hover: {
    bg: "rgba(255,255,255,0.2)",
  },
};
