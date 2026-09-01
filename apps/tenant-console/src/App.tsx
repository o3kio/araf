import { ArafThemeProvider } from "@araf/ui";
import {
  FixtureIdentityProvider,
  FixtureScopeProvider,
  TenantShell,
  TenantRouteGuard,
  type TenantNavigationItem,
  type ProjectOption,
  type RegionOption,
} from "@araf/shell";
import { useState, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate, useLocation, Link } from "react-router";

const projects: ProjectOption[] = [
  { id: "project-alpha", name: "Alpha", organizationId: "org-acme" },
  { id: "project-beta", name: "Beta", organizationId: "org-acme" },
];

const regions: RegionOption[] = [
  { id: "eu-west", name: "EU West" },
  { id: "us-east", name: "US East" },
  { id: "ap-south", name: "AP South" },
];

const navigationItems: TenantNavigationItem[] = [
  { id: "home", type: "link", text: "Home", href: "/" },
  {
    id: "services",
    type: "section",
    text: "Services",
    items: [
      { id: "compute", type: "link", text: "Compute", href: "/services/compute" },
      { id: "networking", type: "link", text: "Networking", href: "/services/networking" },
      { id: "storage", type: "link", text: "Storage", href: "/services/storage" },
      { id: "images", type: "link", text: "Images", href: "/services/images" },
    ],
  },
  {
    id: "manage",
    type: "section",
    text: "Manage",
    items: [
      { id: "operations", type: "link", text: "Operations", href: "/operations" },
      { id: "resources", type: "link", text: "Resources", href: "/resources" },
      { id: "usage", type: "link", text: "Usage & Cost", href: "/usage" },
    ],
  },
  {
    id: "organization",
    type: "section",
    text: "Organization",
    items: [
      { id: "projects", type: "link", text: "Projects", href: "/organization/projects" },
      { id: "users", type: "link", text: "Users & Access", href: "/organization/users" },
      { id: "quotas", type: "link", text: "Quotas", href: "/organization/quotas" },
      { id: "audit", type: "link", text: "Audit", href: "/organization/audit" },
    ],
  },
  {
    id: "developer",
    type: "section",
    text: "Developer",
    items: [{ id: "api", type: "link", text: "API & CLI", href: "/developer/api" }],
  },
];

function HomePage() {
  return (
    <section>
      <h1>Tenant home</h1>
      <p>Welcome to the Araf Tenant Console.</p>
    </section>
  );
}

function PlaceholderPage({ title }: { title: string }) {
  return (
    <section>
      <h1>{title}</h1>
      <p>This page will be implemented in later milestones.</p>
    </section>
  );
}

function NotFound() {
  return (
    <section>
      <h1>Not found</h1>
      <p>The requested tenant page does not exist.</p>
      <Link to="/">Go home</Link>
    </section>
  );
}

function TenantRouterShell({ children }: { children: ReactNode }) {
  const location = useLocation();
  return (
    <TenantShell
      navigationItems={navigationItems}
      activeHref={location.pathname}
      projects={projects}
      regions={regions}
    >
      {children}
    </TenantShell>
  );
}

export function App() {
  const [density] = useState<"comfortable" | "compact">("comfortable");

  return (
    <ArafThemeProvider density={density}>
      <FixtureIdentityProvider
        initialIdentity={{ userId: "fixture-tenant", userName: "Tenant User" }}
      >
        <FixtureScopeProvider
          initialScope={{
            organizationId: "org-acme",
            organizationName: "Acme Corp",
            projectId: "project-alpha",
            projectName: "Alpha",
            regionId: "global",
            regionName: "Global",
          }}
        >
          <BrowserRouter>
            <TenantRouteGuard
              fallback={
                <main style={{ padding: "2rem" }}>
                  <h1>Operator routes are not available in the tenant console</h1>
                  <p>This session cannot access operator-only surfaces.</p>
                </main>
              }
            >
              <Routes>
                <Route
                  path="/"
                  element={
                    <TenantRouterShell>
                      <HomePage />
                    </TenantRouterShell>
                  }
                />
                <Route
                  path="/services/*"
                  element={
                    <TenantRouterShell>
                      <PlaceholderPage title="Services" />
                    </TenantRouterShell>
                  }
                />
                <Route
                  path="/operations"
                  element={
                    <TenantRouterShell>
                      <PlaceholderPage title="Operations" />
                    </TenantRouterShell>
                  }
                />
                <Route
                  path="/resources"
                  element={
                    <TenantRouterShell>
                      <PlaceholderPage title="Resources" />
                    </TenantRouterShell>
                  }
                />
                <Route
                  path="/usage"
                  element={
                    <TenantRouterShell>
                      <PlaceholderPage title="Usage & Cost" />
                    </TenantRouterShell>
                  }
                />
                <Route
                  path="/organization/*"
                  element={
                    <TenantRouterShell>
                      <PlaceholderPage title="Organization" />
                    </TenantRouterShell>
                  }
                />
                <Route
                  path="/developer/*"
                  element={
                    <TenantRouterShell>
                      <PlaceholderPage title="Developer" />
                    </TenantRouterShell>
                  }
                />
                <Route path="/operator/*" element={<Navigate to="/" replace />} />
                <Route
                  path="*"
                  element={
                    <TenantRouterShell>
                      <NotFound />
                    </TenantRouterShell>
                  }
                />
              </Routes>
            </TenantRouteGuard>
          </BrowserRouter>
        </FixtureScopeProvider>
      </FixtureIdentityProvider>
    </ArafThemeProvider>
  );
}
