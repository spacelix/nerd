import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { DashboardPage as App } from "@/App";
import { RouteProvider } from "@/app/RouteContext";
import { ThemeProvider } from "@/app/ThemeProvider";
import "@/styles/index.css";

const rootElement = document.getElementById("root");
if (!rootElement) {
  throw new Error("Root element #root not found");
}

createRoot(rootElement).render(
  <StrictMode>
    <ThemeProvider>
      <RouteProvider>
        <App />
      </RouteProvider>
    </ThemeProvider>
  </StrictMode>,
);
