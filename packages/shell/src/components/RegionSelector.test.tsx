import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { RegionSelector } from "./RegionSelector";

describe("RegionSelector", () => {
  it("includes Global and region options", async () => {
    const onSelect = vi.fn();
    render(
      <RegionSelector
        regions={[{ id: "eu-west", name: "EU West" }]}
        selectedRegionId="global"
        onSelectRegion={onSelect}
      />,
    );

    expect(screen.getByLabelText(/Region/i)).toHaveValue("global");

    await userEvent.selectOptions(screen.getByLabelText(/Region/i), "eu-west");
    expect(onSelect).toHaveBeenCalledWith("eu-west");
  });

  it("selects global explicitly", async () => {
    const onSelect = vi.fn();
    render(
      <RegionSelector
        regions={[{ id: "eu-west", name: "EU West" }]}
        selectedRegionId="eu-west"
        onSelectRegion={onSelect}
      />,
    );

    await userEvent.selectOptions(screen.getByLabelText(/Region/i), "global");
    expect(onSelect).toHaveBeenCalledWith("global");
  });
});
