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

/**
 * Applies Araf theme classes to the document body and renders children.
 * Must be mounted once near the application root.
 */
export function ArafThemeProvider({ density = "comfortable", children }: ArafThemeProviderProps) {
  useEffect(() => {
    const body = document.body;
    const html = document.documentElement;

    html.classList.add(ARAF_THEME_CLASS);

    const densityClass = DENSITY_CLASS[density];
    if (densityClass) {
      body.classList.add(densityClass);
    }

    return () => {
      html.classList.remove(ARAF_THEME_CLASS);
      if (densityClass) {
        body.classList.remove(densityClass);
      }
    };
  }, [density]);

  return <>{children}</>;
}
