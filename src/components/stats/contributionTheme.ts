// Shared visual/text mapping for contribution intensity levels.
export const CONTRIBUTION_LEVEL_COLOR: Record<0 | 1 | 2 | 3 | 4, string> = {
  0: "rgba(148,163,184,0.24)",
  1: "rgba(191, 219, 254, 0.52)",
  2: "rgba(125, 211, 252, 0.62)",
  3: "rgba(45, 212, 191, 0.68)",
  4: "rgba(20, 184, 166, 0.84)",
};

// Human-readable labels shown in the heatmap tooltip.
export const CONTRIBUTION_LEVEL_LABEL: Record<0 | 1 | 2 | 3 | 4, string> = {
  0: "静息",
  1: "微光",
  2: "涌动",
  3: "高燃",
  4: "峰值",
};
