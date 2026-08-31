import { useEffect, type ReactNode } from "react";
import "../tokens.css";
import type { ArafDensity } from "../tokens";
import { COLOR_MODE_CLASS, DENSITY_CLASS } from "../tokens";

export interface ArafThemeProviderProps {
  readonly density?: ArafDensity;
  readonly colorMode?: "light" | "dark";
  readonly children: ReactNode;
}

/**
 * Applies Araf theme classes to the document body and renders children.
 * Must be mounted once near the application root.
 */
export function ArafThemeProvider({
  density = "comfortable",
  colorMode = "light",
  children,
}: ArafThemeProviderProps) {
  useEffect(() => {
    const prevBodyClasses = Array.from(document.body.classList);
    const prevRootClasses = Array.from(document.documentElement.classList);

    const apply = (element: Element, value: string, map: Readonly<Record<string, string>>) => {
      for (const className of Object.values(map)) {
        if (className) {
          element.classList.remove(className);
        }
      }
      const next = map[value];
      if (next) {
        element.classList.add(next);
      }
    };

    apply(document.body, density, DENSITY_CLASS);
    apply(document.documentElement, colorMode, COLOR_MODE_CLASS);

    return () => {
      document.body.className = prevBodyClasses.join(" ");
      document.documentElement.className = prevRootClasses.join(" ");
    };
  }, [density, colorMode]);

  return <>{children}</>;
}
