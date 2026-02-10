import { createSystem, defaultConfig } from "@chakra-ui/react";

export const system = createSystem(defaultConfig, {
  theme: {
    tokens: {
      fonts: {
        heading: { value: "Inter, PingFang SC, Hiragino Sans GB, Microsoft YaHei, sans-serif" },
        body: { value: "Inter, PingFang SC, Hiragino Sans GB, Microsoft YaHei, sans-serif" },
      },
    },
  },
  globalCss: {
    body: {
      bg: "#f8fafc",
      color: "#0f172a",
    },
  },
});
