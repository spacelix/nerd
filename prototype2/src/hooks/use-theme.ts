import { useEffect, useState } from "react";

export type Theme = "dark" | "light" | "system";

const STORAGE_KEY = "nerd.theme";

function getStoredTheme(): Theme {
  if (typeof window === "undefined") return "dark";
  const stored = window.localStorage.getItem(STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "dark";
}

function applyTheme(next: Theme): void {
  const root = document.documentElement;
  const media = window.matchMedia("(prefers-color-scheme: dark)");
  const isDark = next === "dark" || (next === "system" && media.matches);
  root.classList.toggle("dark", isDark);
}

export interface UseThemeResult {
  theme: Theme;
  setTheme: (next: Theme) => void;
  cycle: () => void;
}

export function useTheme(): UseThemeResult {
  const [theme, setThemeState] = useState<Theme>(getStoredTheme);

  useEffect(() => {
    applyTheme(theme);
    window.localStorage.setItem(STORAGE_KEY, theme);
  }, [theme]);

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (): void => {
      if (getStoredTheme() === "system") applyTheme("system");
    };
    media.addEventListener("change", handler);
    return () => media.removeEventListener("change", handler);
  }, []);

  return {
    theme,
    setTheme: setThemeState,
    cycle: () =>
      setThemeState((prev) =>
        prev === "dark" ? "light" : prev === "light" ? "system" : "dark",
      ),
  };
}
