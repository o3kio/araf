import type { RegionOption, RegionId } from "../types";

export interface RegionSelectorProps {
  id?: string;
  label?: string;
  regions: RegionOption[];
  selectedRegionId?: RegionId;
  onSelectRegion: (regionId: RegionId) => void;
  disabled?: boolean;
  globalLabel?: string;
  includeGlobal?: boolean;
}

/**
 * Accessible region selector with explicit `Global` support.
 *
 * The global option uses the literal value `"global"` so it round-trips
 * through URLs and scope state consistently.
 */
export function RegionSelector({
  id = "region-selector",
  label = "Region",
  regions,
  selectedRegionId,
  onSelectRegion,
  disabled = false,
  globalLabel = "Global",
  includeGlobal = true,
}: RegionSelectorProps) {
  const options: RegionOption[] = includeGlobal
    ? [{ id: "global", name: globalLabel }, ...regions]
    : regions;

  return (
    <div className="araf-region-selector">
      <label htmlFor={id} className="araf-region-selector__label">
        {label}
      </label>
      <select
        id={id}
        name="region"
        value={selectedRegionId ?? options[0]?.id ?? ""}
        onChange={(event) => {
          onSelectRegion(event.target.value);
        }}
        disabled={disabled}
        aria-label={label}
      >
        {options.map((region) => (
          <option key={region.id} value={region.id}>
            {region.name}
          </option>
        ))}
      </select>
    </div>
  );
}
