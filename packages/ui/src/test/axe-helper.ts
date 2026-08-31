import { act } from "@testing-library/react";
import { axe } from "jest-axe";

/**
 * Run jest-axe inside React's act() to avoid state-update warnings from
 * Cloudscape's internal effects.
 */
export async function runAxe(container: HTMLElement) {
  let result: Awaited<ReturnType<typeof axe>> | undefined;
  await act(async () => {
    result = await axe(container);
  });
  if (!result) {
    throw new Error("axe did not return a result");
  }
  return result;
}
