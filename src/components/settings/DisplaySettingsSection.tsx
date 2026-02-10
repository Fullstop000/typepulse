import { Box, HStack, Text } from "@chakra-ui/react";
import { ChangeEvent } from "react";
import { MenuBarDisplayMode } from "../../types";
import { useSettingsContext } from "./SettingsContext";

function DisplaySettingsSection() {
  const { snapshot, updateTrayDisplayMode } = useSettingsContext();
  const handleModeChange = (event: ChangeEvent<HTMLSelectElement>) => {
    updateTrayDisplayMode(event.target.value as MenuBarDisplayMode);
  };

  return (
    <Box bg="white" borderRadius="16px" p="6" boxShadow="sm">
      <Text fontSize="xl" fontWeight="semibold" mb="4">展示设置</Text>
      <HStack justify="space-between" align="start" gap="4" flexWrap="wrap">
        <Box>
          <Text fontWeight="semibold">菜单栏显示模式</Text>
          <Text fontSize="sm" color="gray.600">控制菜单栏小组件展示为图标、数字或图标+数字。</Text>
        </Box>
        <select
          value={snapshot.tray_display_mode}
          onChange={handleModeChange}
          style={{
            border: "1px solid #cbd5e1",
            borderRadius: "8px",
            padding: "8px 12px",
            minWidth: "140px",
            background: "white",
          }}
        >
          <option value="icon_only">仅图标</option>
          <option value="text_only">仅数字</option>
          <option value="icon_text">图标 + 数字</option>
        </select>
      </HStack>
    </Box>
  );
}

export default DisplaySettingsSection;
