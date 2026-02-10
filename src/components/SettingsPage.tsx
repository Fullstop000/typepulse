import { Snapshot } from "../types";
import CaptureSettingsSection from "./settings/CaptureSettingsSection";
import DisplaySettingsSection from "./settings/DisplaySettingsSection";
import StorageSettingsSection from "./settings/StorageSettingsSection";
import { SettingSection } from "./settings/types";
import { SettingsProvider } from "./settings/SettingsContext";

type SettingsPageProps = {
  section: SettingSection;
  snapshot: Snapshot;
  onSnapshotChange: (snapshot: Snapshot) => void;
};

function SettingsPage({ section, snapshot, onSnapshotChange }: SettingsPageProps) {
  return (
    <SettingsProvider snapshot={snapshot} onSnapshotChange={onSnapshotChange}>
      {section === "capture" ? <CaptureSettingsSection /> : null}
      {section === "display" ? <DisplaySettingsSection /> : null}
      {section === "storage" ? <StorageSettingsSection /> : null}
    </SettingsProvider>
  );
}

export default SettingsPage;
