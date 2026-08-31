import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { TopNavigation } from "./TopNavigation";

describe("TopNavigation", () => {
  it("renders identity title and link", () => {
    render(<TopNavigation identity={{ title: "Araf", href: "/" }} />);
    expect(screen.getByRole("link", { name: "Araf" })).toHaveAttribute("href", "/");
  });

  it("calls onFollow when identity is activated", async () => {
    const onFollow = vi.fn();
    render(<TopNavigation identity={{ title: "Araf", href: "/", onFollow }} />);
    await userEvent.click(screen.getByRole("link", { name: "Araf" }));
    expect(onFollow).toHaveBeenCalledTimes(1);
  });
});
