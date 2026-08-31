import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { runAxe } from "../test/axe-helper";
import { describe, expect, it, vi } from "vitest";
import { BreadcrumbGroup } from "./BreadcrumbGroup";

describe("BreadcrumbGroup", () => {
  const items = [
    { text: "Home", href: "/" },
    { text: "Resources", href: "/resources" },
  ] as const;

  it("renders breadcrumb items", () => {
    render(<BreadcrumbGroup items={items} />);
    expect(screen.getByRole("link", { name: "Home" })).toBeVisible();
    expect(screen.getByRole("link", { name: "Resources" })).toBeVisible();
  });

  it("calls onFollow with the clicked item", async () => {
    const onFollow = vi.fn();
    render(<BreadcrumbGroup items={items} onFollow={onFollow} />);
    const link = screen.getByRole("link", { name: "Home" });
    await userEvent.click(link);
    expect(onFollow).toHaveBeenCalledWith(items[0]);
  });

  it("exposes a navigation landmark", () => {
    render(<BreadcrumbGroup items={items} />);
    expect(screen.getByRole("navigation")).toBeVisible();
  });

  it("has no accessibility violations", async () => {
    const { container } = render(<BreadcrumbGroup items={items} />);
    expect(await runAxe(container)).toHaveNoViolations();
  });
});
