import { TopNavigation as CloudscapeTopNavigation } from "@cloudscape-design/components";
import type { TopNavigationProps as CloudscapeTopNavigationProps } from "@cloudscape-design/components";

export interface TopNavigationProps {
  readonly identity: {
    readonly title: string;
    readonly href: string;
    readonly logo?: { readonly src: string; readonly alt: string };
    readonly onFollow?: () => void;
  };
  readonly utilities?: CloudscapeTopNavigationProps["utilities"];
  readonly search?: CloudscapeTopNavigationProps["search"];
  readonly i18nStrings?: CloudscapeTopNavigationProps["i18nStrings"];
}

export function TopNavigation({ identity, utilities, search, i18nStrings }: TopNavigationProps) {
  return (
    <CloudscapeTopNavigation
      identity={identity}
      utilities={utilities}
      search={search}
      i18nStrings={i18nStrings}
    />
  );
}
