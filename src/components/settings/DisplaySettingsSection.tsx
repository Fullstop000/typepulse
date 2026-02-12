import { Box, Button, ButtonGroup, HStack, Text } from "@chakra-ui/react";
import { MenuBarDisplayMode } from "../../types";
import { useSettingsContext } from "./SettingsContext";

function DisplaySettingsSection() {
  const { snapshot, updateTrayDisplayMode } = useSettingsContext();
  const handleModeChange = (mode: MenuBarDisplayMode) => updateTrayDisplayMode(mode);

  return (
    <Box bg="#f5f5f6" borderRadius="12px" borderWidth="1px" borderColor="#dddddf" p="0" overflow="hidden">
      <Box px="5" py="4" borderBottomWidth="1px" borderColor="#e4e4e7">
        <Text fontSize="lg" fontWeight="semibold" color="#111827">Appearance</Text>
      </Box>
      <HStack justify="space-between" align="center" gap="4" px="5" py="4" flexWrap="wrap">
        <Box maxW="520px">
          <Text fontWeight="medium" color="#111827" mb="1">菜单栏显示模式</Text>
          <Text fontSize="sm" color="#6b7280">控制菜单栏小组件展示为图标、数字或图标+数字。</Text>
        </Box>
        <ButtonGroup size="sm" variant="outline" attached>
          <Button
            borderColor="#d5d7dc"
            bg={snapshot.tray_display_mode === "icon_only" ? "#ffffff" : "#ebebee"}
            onClick={() => handleModeChange("icon_only")}
          >
            仅图标
          </Button>
          <Button
            borderColor="#d5d7dc"
            bg={snapshot.tray_display_mode === "text_only" ? "#ffffff" : "#ebebee"}
            onClick={() => handleModeChange("text_only")}
          >
            仅数字
          </Button>
          <Button
            borderColor="#d5d7dc"
            bg={snapshot.tray_display_mode === "icon_text" ? "#ffffff" : "#ebebee"}
            onClick={() => handleModeChange("icon_text")}
          >
            图标 + 数字
          </Button>
        </ButtonGroup>
      </HStack>
    </Box>
  );
}

export default DisplaySettingsSection;
