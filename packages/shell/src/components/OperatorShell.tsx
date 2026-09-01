import { AppLayout, TopNavigation, type TopNavigationUtility } from "@araf/ui";
import type { ReactNode } from "react";
import { useIdentity } from "../identity/context";
import "../shell.css";

export interface OperatorNavigationItem {
  id: string;
  type: "section" | "link" | "expandable-link-group";
  text: string;
  href?: string;
  items?: OperatorNavigationItem[];
}

export interface OperatorShellProps {
  children: ReactNode;
  navigationItems: OperatorNavigationItem[];
  activeHref?: string;
}

function renderNavigationItems(items: OperatorNavigationItem[], activeHref?: string): ReactNode {
  return items.map((item) => {
    const isActive = item.href === activeHref;
    if (item.type === "section") {
      return (
        <li key={item.id} className="araf-operator-shell__nav-section">
          <span className="araf-operator-shell__nav-section-title">{item.text}</span>
          {item.items && item.items.length > 0 ? (
            <ul className="araf-operator-shell__nav-sublist" role="list">
              {renderNavigationItems(item.items, activeHref)}
            </ul>
          ) : null}
        </li>
      );
    }

    return (
      <li key={item.id} className="araf-operator-shell__nav-item">
        <a
          href={item.href ?? "#"}
          aria-current={isActive ? "page" : undefined}
          className={`araf-operator-shell__nav-link${isActive ? " is-active" : ""}`}
        >
          {item.text}
        </a>
        {item.items && item.items.length > 0 ? (
          <ul className="araf-operator-shell__nav-sublist" role="list">
            {renderNavigationItems(item.items, activeHref)}
          </ul>
        ) : null}
      </li>
    );
  });
}

/**
 * Operator Console application shell.
 *
 * Provides platform-oriented navigation and a distinct identity from the
 * Tenant Console. It does not expose tenant-only project/region selectors.
 */
export function OperatorShell({ children, navigationItems, activeHref }: OperatorShellProps) {
  const { identity } = useIdentity();

  const utilities: TopNavigationUtility[] = [
    {
      id: "help",
      text: "Help",
      title: "Help",
      ariaLabel: "Help",
      iconName: "help",
      variant: "link",
    },
    {
      id: "user-profile",
      text: identity.userName,
      title: identity.userName,
      ariaLabel: `Operator menu for ${identity.userName}`,
      iconName: "user-profile",
      variant: "link",
    },
  ];

  const navigation = (
    <nav aria-label="Operator navigation">
      <div className="araf-operator-shell__context">
        <span className="araf-operator-shell__context-label" data-testid="operator-context">
          Platform context
        </span>
        <strong className="araf-scope-display__value">O3K control plane</strong>
      </div>
      {navigationItems.length > 0 ? (
        <ul className="araf-operator-shell__nav-list" role="list">
          {renderNavigationItems(navigationItems, activeHref)}
        </ul>
      ) : null}
    </nav>
  );

  return (
    <div className="araf-operator-shell">
      <TopNavigation
        identity={{ title: "Araf Operator", href: "/" }}
        utilities={utilities}
        searchAriaLabel="Search platform"
      />
      <AppLayout navigation={navigation} content={children} />
    </div>
  );
}
