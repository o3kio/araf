import { Tabs as CloudscapeTabs } from "@cloudscape-design/components";
import type { TabsProps as CloudscapeTabsProps } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface TabItem {
  readonly id: string;
  readonly label: string;
  readonly content: ReactNode;
  readonly disabled?: boolean;
  readonly action?: ReactNode;
}

export interface TabsProps {
  readonly tabs: readonly TabItem[];
  readonly activeTabId?: string;
  readonly onChange?: (tabId: string) => void;
  readonly ariaLabel?: string;
}

export function Tabs({ tabs, activeTabId, onChange, ariaLabel }: TabsProps) {
  const cloudscapeTabs: CloudscapeTabsProps["tabs"] = tabs.map((tab) => ({
    id: tab.id,
    label: tab.label,
    content: tab.content,
    disabled: tab.disabled,
    action: tab.action,
  }));

  return (
    <CloudscapeTabs
      tabs={cloudscapeTabs}
      activeTabId={activeTabId}
      onChange={
        onChange
          ? ({ detail }) => {
              onChange(detail.activeTabId);
            }
          : undefined
      }
      ariaLabel={ariaLabel}
    />
  );
}
