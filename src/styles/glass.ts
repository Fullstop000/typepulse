import { SystemStyleObject } from "@chakra-ui/react";

// Shared glass-like surface for top-level cards and panels.
export const glassSurfaceStyle: SystemStyleObject = {
  bg: "glass.bg",
  borderWidth: "1px",
  borderColor: "glass.border",
  backdropFilter: "blur(22px) saturate(1.15)",
  ...( { WebkitBackdropFilter: "blur(22px) saturate(1.15)" } as Record<string, string> ),
  boxShadow: "glass",
};

// Sub-surface for nested blocks inside a glass container.
export const glassSubtleStyle: SystemStyleObject = {
  bg: "glass.subtle",
  borderWidth: "1px",
  borderColor: "glass.borderSoft",
  backdropFilter: "blur(16px) saturate(1.1)",
  ...( { WebkitBackdropFilter: "blur(16px) saturate(1.1)" } as Record<string, string> ),
};

// Pill-style surface for segmented filters and quick actions.
export const glassPillStyle: SystemStyleObject = {
  bg: "glass.pill",
  borderWidth: "1px",
  borderColor: "glass.border",
  backdropFilter: "blur(14px) saturate(1.1)",
  ...( { WebkitBackdropFilter: "blur(14px) saturate(1.1)" } as Record<string, string> ),
};
