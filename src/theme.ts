import { createSystem, defaultConfig } from "@chakra-ui/react";

export const system = createSystem(defaultConfig, {
  theme: {
    tokens: {
      fonts: {
        heading: { value: "-apple-system, BlinkMacSystemFont, Segoe UI, PingFang SC, sans-serif" },
        body: { value: "-apple-system, BlinkMacSystemFont, Segoe UI, PingFang SC, sans-serif" },
      },
    },
  },
  globalCss: {
    body: {
      bg: "#efeff1",
      color: "#1f2328",
    },
  },
});
