/**
 * Araf i18n — lightweight localization framework for the MVP.
 *
 * The MVP ships English only. All user-facing strings should be defined as
 * function calls to `t()` or `format*()` helpers so that a future
 * localization team can replace the string map without touching component
 * logic.
 *
 * Usage:
 *   import { t, formatDate, formatNumber } from "@araf/i18n";
 *   <h1>{t("home.title")}</h1>
 *   <span>{formatDate(new Date())}</span>
 */

// ── String map ────────────────────────────────────────────────────────

/** Key-value map of English-language user-facing strings. */
const en: Record<string, string> = {
  // Navigation
  "nav.home": "Home",
  "nav.operations": "Operations",
  "nav.resources": "Resources",
  "nav.usage": "Usage & Cost",
  "nav.serviceCatalog": "Service catalog",
  "nav.projects": "Projects",
  "nav.users": "Users & Access",
  "nav.quotas": "Quotas",
  "nav.audit": "Audit",
  "nav.api": "API & CLI",

  // Service catalog
  "serviceCatalog.title": "Service catalog",
  "serviceCatalog.empty": "No services are available.",

  // Common actions
  "action.refresh": "Refresh",
  "action.retry": "Retry",
  "action.create": "Create",
  "action.delete": "Delete",
  "action.cancel": "Cancel",

  // Status
  "status.loading": "Loading...",
  "status.error": "Error",

  // Usage
  "usage.title": "Usage & Cost",
  "usage.description":
    "This page shows resource consumption across your project. Cost estimates are shown only when authoritative pricing data is available from the upstream O3K service.",
  "usage.noData": "No usage data",
  "usage.noDataDetail": "No usage records are available for the selected period.",
  "usage.from": "From",
  "usage.to": "To",
  "usage.lastUpdated": "Last updated",

  // Quota
  "quota.title": "Quota overview",
  "quota.noData": "No quota data",
  "quota.noDataDetail": "No quota information is available.",
};

/** Look up a localized string by key. Falls back to the key itself. */
export function t(key: string): string {
  return en[key] ?? key;
}

/** Look up a localized string and interpolate `{n}` placeholders. */
export function tpl(key: string, params: Record<string, string | number>): string {
  let s = t(key);
  for (const [k, v] of Object.entries(params)) {
    s = s.replace(`{${k}}`, String(v));
  }
  return s;
}

// ── Date / time / number formatting ───────────────────────────────────

const dateFormatter = new Intl.DateTimeFormat("en-US", {
  dateStyle: "medium",
  timeStyle: "short",
  timeZone: "UTC",
});

const numberFormatter = new Intl.NumberFormat("en-US");

/** Format a Date or ISO string as locale-safe date+time. */
export function formatDate(value: Date | string): string {
  const d = typeof value === "string" ? new Date(value) : value;
  return dateFormatter.format(d);
}

/** Format a number with locale-safe digit grouping. */
export function formatNumber(value: number): string {
  return numberFormatter.format(value);
}

/** Format a number as a percentage string (e.g. "75%"). */
export function formatPercent(value: number): string {
  return `${numberFormatter.format(value)}%`;
}
