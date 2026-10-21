import { FrontsPage } from "./features/planets/FrontsPage";
import { HistoryPage } from "./features/history/HistoryPage";
import { MapPage } from "./features/map/MapPage";
import { NewsPage } from "./features/news/NewsPage";
import { OrdersPage } from "./features/major-order/MajorOrder";
import { StatsPage } from "./features/stats/StatsPage";
import { Shell } from "./layout/Shell";
import { useUi } from "./state";
import { useDataSync } from "./sync";

export default function App() {
  useDataSync();
  const view = useUi((s) => s.view);
  return (
    <Shell>
      {view === "map" && <MapPage />}
      {view === "fronts" && <FrontsPage />}
      {view === "orders" && <OrdersPage />}
      {view === "news" && <NewsPage />}
      {view === "stats" && <StatsPage />}
      {view === "history" && <HistoryPage />}
    </Shell>
  );
}
