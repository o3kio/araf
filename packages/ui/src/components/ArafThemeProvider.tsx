import { useEffect, type ReactNode } from "react";
import "@cloudscape-design/global-styles/index.css";
import "../tokens.css";
import type { ArafDensity } from "../tokens";
import { DENSITY_CLASS } from "../tokens";

export interface ArafThemeProviderProps {
  readonly density?: ArafDensity;
  readonly children: ReactNode;
}

const ARAF_THEME_CLASS = "araf-theme";

let arafThemeRefCount = 0;

/**
 * Applies Araf theme classes to the document body and renders children.
 * Must be mounted once near the application root.
 *
 * A module-level ref-count keeps the theme class alive if multiple providers
 * are mounted (e.g. during React StrictMode double-mount or micro-frontends),
 * and only removes it when the last provider unmounts.
 */
export function ArafThemeProvider({ density = "comfortable", children }: ArafThemeProviderProps) {
  useEffect(() => {
    const body = document.body;
    const html = document.documentElement;

    arafThemeRefCount += 1;
    html.classList.add(ARAF_THEME_CLASS);

    const densityClass = DENSITY_CLASS[density];
    if (densityClass) {
      body.classList.add(densityClass);
    }

    return () => {
      if (densityClass) {
        body.classList.remove(densityClass);
      }
      arafThemeRefCount -= 1;
      if (arafThemeRefCount <= 0) {
        html.classList.remove(ARAF_THEME_CLASS);
      }
    };
  }, [density]);

  return <>{children}</>;
}
