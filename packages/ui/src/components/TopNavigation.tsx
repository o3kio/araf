import {
  TopNavigation as CloudscapeTopNavigation,
  type IconProps,
  type TopNavigationProps as CloudscapeTopNavigationProps,
} from "@cloudscape-design/components";
import type { ReactNode } from "react";

/** Araf icon name vocabulary. Maps to Cloudscape icon names internally. */
export type TopNavigationIconName = "notification" | "settings" | "user-profile" | "help" | "menu";

const iconNameMap = {
  notification: "notification",
  settings: "settings",
  "user-profile": "user-profile",
  help: "help",
  menu: "menu",
} as const satisfies Record<TopNavigationIconName, string>;

/** Araf top-navigation button utility. Narrowed to the actions Araf shells need. */
export interface TopNavigationUtility {
  readonly id: string;
  readonly text?: string;
  readonly title?: string;
  readonly iconName?: TopNavigationIconName;
  readonly ariaLabel?: string;
  readonly badge?: boolean;
  readonly variant?: "primary" | "link";
  readonly href?: string;
  readonly external?: boolean;
  readonly onPress?: () => void;
}

export interface TopNavigationProps {
  readonly identity: {
    readonly title: string;
    readonly href: string;
    readonly logo?: { readonly src: string; readonly alt: string };
    readonly onPress?: () => void;
  };
  readonly utilities?: readonly TopNavigationUtility[];
  readonly search?: ReactNode;
  readonly searchAriaLabel?: string;
}

function mapUtilities(
  utilities: readonly TopNavigationUtility[],
): CloudscapeTopNavigationProps["utilities"] {
  return utilities.map((utility) => ({
    id: utility.id,
    type: "button" as const,
    text: utility.text,
    title: utility.title,
    ariaLabel: utility.ariaLabel,
    badge: utility.badge,
    iconName: utility.iconName ? (iconNameMap[utility.iconName] as IconProps.Name) : undefined,
    variant: utility.variant === "primary" ? "primary-button" : "link",
    href: utility.href,
    external: utility.external,
    onClick: utility.onPress,
  }));
}

export function TopNavigation({
  identity,
  utilities,
  search,
  searchAriaLabel = "Search",
}: TopNavigationProps) {
  const cloudscapeIdentity: CloudscapeTopNavigationProps["identity"] = {
    title: identity.title,
    href: identity.href,
    logo: identity.logo,
    onFollow: identity.onPress
      ? () => {
          const handler = identity.onPress;
          if (handler) handler();
        }
      : undefined,
  };

  return (
    <CloudscapeTopNavigation
      identity={cloudscapeIdentity}
      utilities={utilities ? mapUtilities(utilities) : undefined}
      search={search}
      i18nStrings={{ searchIconAriaLabel: searchAriaLabel }}
    />
  );
}
