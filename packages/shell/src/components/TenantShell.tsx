import { AppLayout, TopNavigation, type TopNavigationUtility } from "@araf/ui";
import type { ReactNode } from "react";
import { useScope } from "../scope/context";
import { useIdentity } from "../identity/context";
import { ProjectSelector } from "./ProjectSelector";
import { RegionSelector } from "./RegionSelector";
import { ScopeDisplay } from "./ScopeDisplay";
import type { ProjectOption, RegionOption } from "../types";
import "../shell.css";

export interface TenantNavigationItem {
  id: string;
  type: "section" | "link" | "expandable-link-group";
  text: string;
  href?: string;
  items?: TenantNavigationItem[];
}

export interface TenantShellProps {
  children: ReactNode;
  navigationItems: TenantNavigationItem[];
  activeHref?: string;
  projects: ProjectOption[];
  regions: RegionOption[];
  operationsHref?: string;
}

function flattenNavItems(items: TenantNavigationItem[]): { href: string; text: string }[] {
  const result: { href: string; text: string }[] = [];
  for (const item of items) {
    if (item.href) {
      result.push({ href: item.href, text: item.text });
    }
    if (item.items) {
      result.push(...flattenNavItems(item.items));
    }
  }
  return result;
}

function renderNavigationItems(items: TenantNavigationItem[], activeHref?: string): ReactNode {
  return items.map((item) => {
    const isActive = item.href === activeHref;
    if (item.type === "section") {
      return (
        <li key={item.id} className="araf-tenant-shell__nav-section">
          <span className="araf-tenant-shell__nav-section-title">{item.text}</span>
          {item.items && item.items.length > 0 ? (
            <ul className="araf-tenant-shell__nav-sublist" role="list">
              {renderNavigationItems(item.items, activeHref)}
            </ul>
          ) : null}
        </li>
      );
    }

    return (
      <li key={item.id} className="araf-tenant-shell__nav-item">
        <a
          href={item.href ?? "#"}
          aria-current={isActive ? "page" : undefined}
          className={`araf-tenant-shell__nav-link${isActive ? " is-active" : ""}`}
        >
          {item.text}
        </a>
        {item.items && item.items.length > 0 ? (
          <ul className="araf-tenant-shell__nav-sublist" role="list">
            {renderNavigationItems(item.items, activeHref)}
          </ul>
        ) : null}
      </li>
    );
  });
}

/**
 * Tenant Console application shell.
 *
 * Permanently surfaces project/region scope and provides structural tenant
 * navigation. The operator route table is not reachable from this shell.
 */
export function TenantShell({
  children,
  navigationItems,
  activeHref,
  projects,
  regions,
  operationsHref = "/operations",
}: TenantShellProps) {
  const { scope, setScope } = useScope();
  const { identity } = useIdentity();

  const navLinks = flattenNavItems(navigationItems);
  const hasOperations = navLinks.some((link) => link.href === operationsHref);

  const utilities: TopNavigationUtility[] = [
    {
      id: "operations",
      text: "Operations",
      title: "Operations",
      ariaLabel: "Operations",
      href: operationsHref,
      variant: "link",
    },
    {
      id: "notifications",
      text: "Notifications",
      title: "Notifications",
      ariaLabel: "Notifications",
      iconName: "notification",
      badge: false,
      variant: "link",
    },
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
      ariaLabel: `User menu for ${identity.userName}`,
      iconName: "user-profile",
      variant: "link",
    },
  ];

  const navigation = (
    <nav aria-label="Tenant navigation">
      <div className="araf-tenant-shell__scope">
        <ScopeDisplay label="Current tenant scope" />
        <ProjectSelector
          id="tenant-project-selector"
          label="Project"
          projects={projects}
          selectedProjectId={scope.projectId}
          onSelectProject={(projectId) => {
            const selected = projects.find((p) => p.id === projectId);
            setScope((prev) => ({
              ...prev,
              projectId,
              projectName: selected?.name,
            }));
          }}
        />
        <RegionSelector
          id="tenant-region-selector"
          label="Region"
          regions={regions}
          selectedRegionId={scope.regionId ?? "global"}
          onSelectRegion={(regionId) => {
            const selected = regions.find((r) => r.id === regionId);
            setScope((prev) => ({
              ...prev,
              regionId,
              regionName: selected?.name,
            }));
          }}
        />
      </div>
      {navigationItems.length > 0 ? (
        <ul className="araf-tenant-shell__nav-list" role="list">
          {renderNavigationItems(navigationItems, activeHref)}
        </ul>
      ) : null}
      {!hasOperations ? (
        <div className="araf-tenant-shell__operations-entry">
          <a href={operationsHref} className="araf-tenant-shell__nav-link">
            Operations
          </a>
        </div>
      ) : null}
    </nav>
  );

  return (
    <div className="araf-tenant-shell">
      <TopNavigation
        identity={{ title: "Araf Tenant", href: "/" }}
        utilities={utilities}
        search={
          <input
            className="araf-tenant-shell__search"
            type="search"
            placeholder="Search resources"
            aria-label="Search resources"
          />
        }
        searchAriaLabel="Search resources"
      />
      <AppLayout navigation={navigation} content={children} />
    </div>
  );
}
