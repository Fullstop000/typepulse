import { createSystem, defaultConfig } from "@chakra-ui/react";

export const system = createSystem(defaultConfig, {
  theme: {
    tokens: {
      fonts: {
        heading: { value: "-apple-system, BlinkMacSystemFont, Segoe UI, PingFang SC, sans-serif" },
        body: { value: "-apple-system, BlinkMacSystemFont, Segoe UI, PingFang SC, sans-serif" },
      },
      colors: {
        glass: {
          bg: { value: "rgba(255, 255, 255, 0.34)" },
          subtle: { value: "rgba(255, 255, 255, 0.24)" },
          pill: { value: "rgba(255, 255, 255, 0.3)" },
          border: { value: "rgba(255, 255, 255, 0.58)" },
          borderSoft: { value: "rgba(255, 255, 255, 0.42)" },
          hover: { value: "rgba(255, 255, 255, 0.8)" },
        },
      },
      shadows: {
        glass: {
          value:
            "0 10px 26px rgba(15, 23, 42, 0.08), 0 2px 8px rgba(15, 23, 42, 0.04), inset 0 1px 0 rgba(255, 255, 255, 0.58)",
        },
      },
    },
  },
  globalCss: {
    body: {
      bg: `
        radial-gradient(920px 500px at 14% -10%, rgba(191, 219, 254, 0.24), transparent 60%),
        radial-gradient(760px 440px at 86% 6%, rgba(199, 210, 254, 0.2), transparent 56%),
        radial-gradient(860px 560px at 52% 118%, rgba(203, 213, 225, 0.2), transparent 60%),
        linear-gradient(135deg, #ecf1f8 0%, #f2f6fb 52%, #eaf0f8 100%)
      `,
      color: "#1f2328",
      minH: "100vh",
      backgroundAttachment: "fixed",
    },
    "#root": {
      minH: "100vh",
    },
  },
});
