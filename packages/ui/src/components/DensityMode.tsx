import { Toggle } from "@cloudscape-design/components";
import type { ArafDensity } from "../tokens";

export interface DensityModeProps {
  readonly density: ArafDensity;
  readonly onChange: (density: ArafDensity) => void;
}

export function DensityMode({ density, onChange }: DensityModeProps) {
  const checked = density === "compact";
  return (
    <Toggle
      checked={checked}
      onChange={({ detail }) => {
        onChange(detail.checked ? "compact" : "comfortable");
      }}
    >
      {density === "compact" ? "Compact" : "Comfortable"}
    </Toggle>
  );
}
