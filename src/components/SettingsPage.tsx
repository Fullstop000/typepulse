import { Stack } from "@chakra-ui/react";
import { Snapshot } from "../types";
import CaptureSettingsSection from "./settings/CaptureSettingsSection";
import DisplaySettingsSection from "./settings/DisplaySettingsSection";
import { SettingsProvider } from "./settings/SettingsContext";
import StorageSettingsSection from "./settings/StorageSettingsSection";
import { SettingSection } from "./settings/types";

type SettingsPageProps = {
  section: SettingSection;
  snapshot: Snapshot;
  onSnapshotChange: (snapshot: Snapshot) => void;
};

function SettingsPage({ section, snapshot, onSnapshotChange }: SettingsPageProps) {
  return (
    <SettingsProvider snapshot={snapshot} onSnapshotChange={onSnapshotChange}>
      <Stack gap="4">
        {section === "capture" ? <CaptureSettingsSection /> : null}
        {section === "display" ? <DisplaySettingsSection /> : null}
        {section === "storage" ? <StorageSettingsSection /> : null}
      </Stack>
    </SettingsProvider>
  );
}

export default SettingsPage;
