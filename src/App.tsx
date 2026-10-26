import { CampaignsPage } from "./features/campaigns/CampaignsPage";
import { HellpadPage } from "./features/hellpad/HellpadPage";
import { MapPage } from "./features/map/MapPage";
import { NewsPage } from "./features/news/NewsPage";
import { OverviewPage } from "./features/overview/OverviewPage";
import { SettingsPage } from "./features/settings/SettingsPage";
import { Shell } from "./layout/Shell";
import { useNavHistory } from "./navHistory";
import { useUi } from "./state";
import { useDataSync } from "./sync";
import { useCloseToTray } from "./tray";

export default function App() {
  useDataSync();
  useNavHistory();
  useCloseToTray();
  const view = useUi((s) => s.view);
  return (
    <Shell>
      {view === "campaigns" && <CampaignsPage />}
      {view === "map" && <MapPage />}
      {view === "hellpad" && <HellpadPage />}
      {view === "overview" && <OverviewPage />}
      {view === "news" && <NewsPage />}
      {view === "settings" && <SettingsPage />}
    </Shell>
  );
}
