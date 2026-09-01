import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ProjectSelector } from "./ProjectSelector";

describe("ProjectSelector", () => {
  it("renders options and selects via keyboard", async () => {
    const onSelect = vi.fn();
    render(
      <ProjectSelector
        projects={[
          { id: "p1", name: "Alpha" },
          { id: "p2", name: "Beta" },
        ]}
        selectedProjectId="p1"
        onSelectProject={onSelect}
      />,
    );

    expect(screen.getByLabelText(/Project/i)).toHaveValue("p1");

    await userEvent.selectOptions(screen.getByLabelText(/Project/i), "p2");
    expect(onSelect).toHaveBeenCalledWith("p2");
  });

  it("is disabled when there are no projects", () => {
    render(<ProjectSelector projects={[]} onSelectProject={() => undefined} />);
    expect(screen.getByLabelText(/Project/i)).toBeDisabled();
  });
});
