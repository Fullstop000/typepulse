import { Box, Button, HStack, Stack, Text } from "@chakra-ui/react";
import { Snapshot } from "../../../types";
import CaptureSettingsSection from "../CaptureSettingsSection";
import DisplaySettingsSection from "../DisplaySettingsSection";
import { SettingsProvider } from "../SettingsContext";
import StorageSettingsSection from "../StorageSettingsSection";

type SettingsPageProps = {
  snapshot: Snapshot;
  onSnapshotChange: (snapshot: Snapshot) => void;
};

// Scroll to section anchor in settings one-page layout.
function scrollToAnchor(anchorId: string) {
  const section = document.getElementById(anchorId);
  if (!section) {
    return;
  }
  section.scrollIntoView({ behavior: "smooth", block: "start" });
}

function SettingsPage({ snapshot, onSnapshotChange }: SettingsPageProps) {
  return (
    <SettingsProvider snapshot={snapshot} onSnapshotChange={onSnapshotChange}>
      <Stack gap="4">
        <HStack justify="space-between" align="center">
          <Text fontSize="2xl" fontWeight="semibold" color="gray.800">
            Settings
          </Text>
          <HStack
            gap="1"
            bg="glass.pill"
            borderWidth="1px"
            borderColor="glass.border"
            backdropFilter="blur(10px) saturate(1.05)"
            css={{ WebkitBackdropFilter: "blur(10px) saturate(1.05)" }}
            borderRadius="999px"
            p="1"
          >
            <Button
              size="sm"
              variant="ghost"
              borderRadius="999px"
              bg="rgba(255,255,255,0.68)"
              _hover={{ bg: "rgba(255,255,255,0.8)" }}
              onClick={() => scrollToAnchor("settings-general")}
            >
              General
            </Button>
            <Button
              size="sm"
              variant="ghost"
              borderRadius="999px"
              bg="rgba(255,255,255,0.68)"
              _hover={{ bg: "rgba(255,255,255,0.8)" }}
              onClick={() => scrollToAnchor("settings-appearance")}
            >
              Appearance
            </Button>
            <Button
              size="sm"
              variant="ghost"
              borderRadius="999px"
              bg="rgba(255,255,255,0.68)"
              _hover={{ bg: "rgba(255,255,255,0.8)" }}
              onClick={() => scrollToAnchor("settings-storage")}
            >
              Storage
            </Button>
          </HStack>
        </HStack>

        <Box id="settings-general">
          <CaptureSettingsSection />
        </Box>
        <Box id="settings-appearance">
          <DisplaySettingsSection />
        </Box>
        <Box id="settings-storage">
          <StorageSettingsSection />
        </Box>
      </Stack>
    </SettingsProvider>
  );
}

export default SettingsPage;
