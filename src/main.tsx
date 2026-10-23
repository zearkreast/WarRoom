import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import React from "react";
import ReactDOM from "react-dom/client";
import "d3-transition";
import "@fontsource/chakra-petch/400.css";
import "@fontsource/chakra-petch/500.css";
import "@fontsource/chakra-petch/600.css";
import "@fontsource/chakra-petch/700.css";
import "@fontsource-variable/archivo/wdth.css";
import "./ui/theme.css";
import "./ui/ui.css";
import App from "./App";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      // Keep showing the last good data while Supabase is unreachable.
      refetchOnWindowFocus: false,
      refetchIntervalInBackground: true,
      gcTime: Infinity,
    },
  },
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <App />
    </QueryClientProvider>
  </React.StrictMode>,
);
