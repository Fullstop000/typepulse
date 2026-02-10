import { Stack } from "@chakra-ui/react";
import { Snapshot } from "../../../types";
import CaptureSettingsSection from "../CaptureSettingsSection";
import DisplaySettingsSection from "../DisplaySettingsSection";
import { SettingsProvider } from "../SettingsContext";
import StorageSettingsSection from "../StorageSettingsSection";
import { SettingSection } from "../types";

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
