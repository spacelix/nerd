import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { DEFAULT_ROUTE, isRouteId, type RouteId } from "./router";

const STORAGE_KEY = "nerd-prototype-route";
const SELECTED_KEY = "nerd-prototype-selected-project";

type RouteContextValue = {
  route: RouteId;
  setRoute: (id: RouteId) => void;
  selectedProjectId: string | null;
  setSelectedProjectId: (id: string | null) => void;
};

const RouteContext = createContext<RouteContextValue | null>(null);

function readStoredRoute(): RouteId {
  if (typeof window === "undefined") return DEFAULT_ROUTE;
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored && isRouteId(stored)) return stored;
  return DEFAULT_ROUTE;
}

function readStoredSelected(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(SELECTED_KEY);
}

export function RouteProvider({ children }: { children: ReactNode }) {
  const [route, setRouteState] = useState<RouteId>(() => readStoredRoute());
  const [selectedProjectId, setSelectedProjectIdState] = useState<string | null>(
    () => readStoredSelected(),
  );

  const setRoute = useCallback((id: RouteId) => {
    setRouteState(id);
    if (typeof window !== "undefined") {
      window.localStorage.setItem(STORAGE_KEY, id);
    }
  }, []);

  const setSelectedProjectId = useCallback((id: string | null) => {
    setSelectedProjectIdState(id);
    if (typeof window !== "undefined") {
      if (id) window.localStorage.setItem(SELECTED_KEY, id);
      else window.localStorage.removeItem(SELECTED_KEY);
    }
  }, []);

  useEffect(() => {
    if (typeof window === "undefined") return;
    window.localStorage.setItem(STORAGE_KEY, route);
  }, [route]);

  const value = useMemo<RouteContextValue>(
    () => ({ route, setRoute, selectedProjectId, setSelectedProjectId }),
    [route, setRoute, selectedProjectId, setSelectedProjectId],
  );

  return (
    <RouteContext.Provider value={value}>{children}</RouteContext.Provider>
  );
}

export function useRoute(): RouteContextValue {
  const ctx = useContext(RouteContext);
  if (!ctx) throw new Error("useRoute must be used within RouteProvider");
  return ctx;
}
