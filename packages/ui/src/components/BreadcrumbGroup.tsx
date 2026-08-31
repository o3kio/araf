import { BreadcrumbGroup as CloudscapeBreadcrumbGroup } from "@cloudscape-design/components";
import type { BreadcrumbGroupProps as CloudscapeBreadcrumbGroupProps } from "@cloudscape-design/components";

export interface BreadcrumbItem {
  readonly text: string;
  readonly href: string;
}

export interface BreadcrumbGroupProps {
  readonly items: readonly BreadcrumbItem[];
  readonly onFollow?: (item: BreadcrumbItem) => void;
  readonly ariaLabel?: string;
}

export function BreadcrumbGroup({
  items,
  onFollow,
  ariaLabel = "Breadcrumbs",
}: BreadcrumbGroupProps) {
  const cloudscapeItems: CloudscapeBreadcrumbGroupProps["items"] = items.map((item) => ({
    text: item.text,
    href: item.href,
  }));

  return (
    <CloudscapeBreadcrumbGroup
      items={cloudscapeItems}
      ariaLabel={ariaLabel}
      onFollow={
        onFollow
          ? (e) => {
              e.preventDefault();
              const match = items.find((i) => i.href === e.detail.href);
              if (match) onFollow(match);
            }
          : undefined
      }
    />
  );
}
