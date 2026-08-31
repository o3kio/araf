/**
 * Araf design tokens — semantic layer over Cloudscape design tokens.
 *
 * This module is the **only** place Cloudscape tokens are imported.
 * Application code consumes Araf token objects, never Cloudscape tokens
 * directly (ADR 0004).
 */

import {
  borderRadiusInput,
  borderRadiusItem,
  borderRadiusContainer,
  colorTextStatusError,
  colorTextStatusInactive,
  colorTextStatusInfo,
  colorTextStatusSuccess,
  colorTextStatusWarning,
  fontSizeBodyS,
  fontSizeBodyM,
  fontSizeHeadingS,
  fontSizeHeadingM,
  fontSizeHeadingL,
  fontSizeHeadingXl,
  fontSizeDisplayL,
  spaceScaledXxxs,
  spaceScaledXxs,
  spaceScaledXs,
  spaceScaledS,
  spaceScaledM,
  spaceScaledL,
  spaceScaledXl,
  spaceScaledXxl,
  spaceScaledXxxl,
} from "@cloudscape-design/design-tokens";

export type ArafDensity = "comfortable" | "compact";

export interface ArafTheme {
  density: ArafDensity;
  colorMode: "light" | "dark";
}

/** Opt-in CSS class for compact mode. Applied by ArafThemeProvider. */
export const DENSITY_CLASS = {
  comfortable: "",
  compact: "araf-density-compact",
} as const satisfies Record<ArafDensity, string>;

/** Opt-in body class for dark mode. */
export const COLOR_MODE_CLASS = {
  light: "",
  dark: "araf-color-mode-dark",
} as const satisfies Record<string, string>;

/**
 * Araf semantic status colors. Backed by Cloudscape design tokens but
 * exposed through this mapping so consumers never import Cloudscape tokens.
 */
export const statusColors = {
  success: colorTextStatusSuccess,
  error: colorTextStatusError,
  warning: colorTextStatusWarning,
  info: colorTextStatusInfo,
  pending: colorTextStatusInactive,
} as const;

/**
 * Araf spacing scale (multiples of the base unit 4px, with semantic names).
 */
export const spacing = {
  xxxs: spaceScaledXxxs,
  xxs: spaceScaledXxs,
  xs: spaceScaledXs,
  s: spaceScaledS,
  m: spaceScaledM,
  l: spaceScaledL,
  xl: spaceScaledXl,
  xxl: spaceScaledXxl,
  xxxl: spaceScaledXxxl,
} as const;

/**
 * Araf font-size scale.
 */
export const fontSize = {
  bodyS: fontSizeBodyS,
  bodyM: fontSizeBodyM,
  headingS: fontSizeHeadingS,
  headingM: fontSizeHeadingM,
  headingL: fontSizeHeadingL,
  headingXl: fontSizeHeadingXl,
  displayL: fontSizeDisplayL,
} as const;

/**
 * Araf font-weight scale.
 */
export const fontWeight = {
  regular: "400",
  bold: "700",
} as const;

/**
 * Araf border-radius scale.
 */
export const borderRadius = {
  none: "0",
  xs: borderRadiusInput,
  s: "var(--border-radius-control, 6px)",
  m: borderRadiusItem,
  l: borderRadiusContainer,
  pill: "9999px",
} as const;

/**
 * Araf focus ring style (visible keyboard focus per WCAG 2.2).
 */
export const focus = {
  outline: "2px solid var(--color-border-focused, #0972d3)",
  outlineOffset: "2px",
} as const;

/**
 * Motion duration constraints (calm enterprise — no gratuitous animation).
 */
export const motion = {
  durationFast: "90ms",
  durationNormal: "135ms",
  durationSlow: "180ms",
  easing: "ease-in-out",
} as const;
