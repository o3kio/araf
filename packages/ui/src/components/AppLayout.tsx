import { AppLayout as CloudscapeAppLayout } from "@cloudscape-design/components";
import type { AppLayoutProps as CloudscapeAppLayoutProps } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface AppLayoutProps {
  readonly navigation?: ReactNode;
  readonly content: ReactNode;
  readonly tools?: ReactNode;
  readonly breadcrumbs?: ReactNode;
  readonly minContentWidth?: number;
  readonly maxContentWidth?: number;
}

const defaultAriaLabels: CloudscapeAppLayoutProps["ariaLabels"] = {
  navigation: "Navigation drawer",
  navigationClose: "Close navigation drawer",
  navigationToggle: "Open navigation drawer",
  notifications: "Notifications",
  tools: "Tools drawer",
  toolsClose: "Close tools drawer",
  toolsToggle: "Open tools drawer",
  drawers: "Drawers",
  drawersOverflow: "Overflow drawers",
  drawersOverflowWithBadge: "Overflow drawers (Unread notifications)",
};

export function AppLayout({
  navigation,
  content,
  tools,
  breadcrumbs,
  minContentWidth = 700,
  maxContentWidth = 1400,
}: AppLayoutProps) {
  const nav: CloudscapeAppLayoutProps["navigation"] = navigation ? <>{navigation}</> : undefined;
  const toolsSlot: CloudscapeAppLayoutProps["tools"] = tools ? <>{tools}</> : undefined;

  return (
    <CloudscapeAppLayout
      navigation={nav}
      content={<>{content}</>}
      tools={toolsSlot}
      breadcrumbs={breadcrumbs ? <>{breadcrumbs}</> : undefined}
      minContentWidth={minContentWidth}
      maxContentWidth={maxContentWidth}
      ariaLabels={defaultAriaLabels}
    />
  );
}
