import { Toggle } from "@cloudscape-design/components";
import type { ArafDensity } from "../tokens";

export interface DensityModeProps {
  readonly density: ArafDensity;
  readonly onChange: (density: ArafDensity) => void;
}

/**
 * Density toggle.
 *
 * The control label is stable ("Compact mode"); the checked state and the
 * adjacent value text convey the current density. This keeps the accessible
 * name predictable while still exposing the active value visually.
 */
export function DensityMode({ density, onChange }: DensityModeProps) {
  const checked = density === "compact";
  return (
    <Toggle
      checked={checked}
      onChange={({ detail }) => {
        onChange(detail.checked ? "compact" : "comfortable");
      }}
    >
      Compact mode <span aria-hidden="true">({checked ? "Compact" : "Comfortable"})</span>
    </Toggle>
  );
}
