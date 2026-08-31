import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it, vi } from "vitest";
import { Tabs } from "./Tabs";

describe("Tabs", () => {
  const tabs = [
    { id: "overview", label: "Overview", content: <p>Overview content</p> },
    { id: "config", label: "Configuration", content: <p>Config content</p> },
  ] as const;

  it("renders tab labels", () => {
    render(<Tabs tabs={tabs} activeTabId="overview" onChange={vi.fn()} />);
    expect(screen.getByRole("tab", { name: "Overview" })).toBeVisible();
    expect(screen.getByRole("tab", { name: "Configuration" })).toBeVisible();
  });

  it("displays active tab content", () => {
    render(<Tabs tabs={tabs} activeTabId="config" onChange={vi.fn()} />);
    expect(screen.getByText("Config content")).toBeVisible();
  });

  it("calls onChange when a tab is selected", async () => {
    const onChange = vi.fn();
    render(<Tabs tabs={tabs} activeTabId="overview" onChange={onChange} />);
    await userEvent.click(screen.getByRole("tab", { name: "Configuration" }));
    expect(onChange).toHaveBeenCalledWith("config");
  });

  it("has no accessibility violations", async () => {
    const { container } = render(
      <Tabs tabs={tabs} activeTabId="overview" onChange={vi.fn()} ariaLabel="Sections" />,
    );
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
