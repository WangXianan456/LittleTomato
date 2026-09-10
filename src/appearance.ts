import { useEffect, useState } from "react";
import type { Settings } from "./settings";
export function useColorTheme(theme: Settings["theme"]) {
  const [dark, setDark] = useState(() => matchMedia("(prefers-color-scheme: dark)").matches);
  useEffect(() => {
    const media = matchMedia("(prefers-color-scheme: dark)");
    const change = () => setDark(media.matches);
    media.addEventListener("change", change);
    return () => media.removeEventListener("change", change);
  }, []);
  return theme === "system" ? (dark ? "dark" : "light") : theme;
}
