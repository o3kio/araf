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
import {
  ResourceClientProvider,
  ResourceLandingPage,
  ResourceCollectionPage,
  ResourceDetailPage,
  ResourceCreatePage,
} from "@araf/resources";
import {
  OperationsClientProvider,
  OperationsListPage,
  OperationDetailPage,
} from "@araf/operations";
import {
  GovernanceClientProvider,
  ProjectsPage,
  ProjectDetailPage,
  UsersPage,
  UserDetailPage,
  QuotasPage,
  AuditPage,
  ApiCredentialsPage,
} from "@araf/governance";
import { createArafClient } from "@araf/api-client";
import { useState, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate, useLocation, Link, useParams } from "react-router";

const projects: ProjectOption[] = [
  { id: "project-1", name: "Project 1", organizationId: "org-acme" },
  { id: "project-2", name: "Project 2", organizationId: "org-acme" },
  { id: "project-3", name: "Project 3", organizationId: "org-acme" },
  { id: "project-4", name: "Project 4", organizationId: "org-acme" },
  { id: "project-5", name: "Project 5", organizationId: "org-acme" },
];

const regions: RegionOption[] = [
  { id: "eu-west", name: "EU West" },
  { id: "us-east", name: "US East" },
  { id: "ap-south", name: "AP South" },
];

const tenantBffUrl =
  (import.meta.env.VITE_TENANT_BFF_URL as string | undefined) ?? "http://127.0.0.1:8080";
const arafClient = createArafClient(tenantBffUrl);

const navigationItems: TenantNavigationItem[] = [
  { id: "home", type: "link", text: "Home", href: "/" },
  {
    id: "services",
    type: "section",
    text: "Services",
    items: [
      { id: "compute", type: "link", text: "Compute", href: "/resources/compute.server" },
      { id: "networking", type: "link", text: "Networking", href: "/resources/network.vpc" },
      { id: "storage", type: "link", text: "Storage", href: "/resources/storage.volume" },
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

function ResourceCollectionRoute() {
  const { resourceType } = useParams<{ resourceType: string }>();
  return <ResourceCollectionPage resourceType={decodeURIComponent(resourceType ?? "")} />;
}

function ResourceDetailRoute() {
  const { resourceType } = useParams<{ resourceType: string }>();
  return <ResourceDetailPage resourceType={decodeURIComponent(resourceType ?? "")} />;
}

function ResourceCreateRoute() {
  const { resourceType } = useParams<{ resourceType: string }>();
  return <ResourceCreatePage resourceType={decodeURIComponent(resourceType ?? "")} />;
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
            projectId: "project-1",
            projectName: "Project 1",
            regionId: "eu-west",
            regionName: "EU West",
          }}
        >
          <ResourceClientProvider client={arafClient}>
            <OperationsClientProvider client={arafClient}>
              <GovernanceClientProvider client={arafClient}>
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
                            <OperationsListPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/operations/:id"
                        element={
                          <TenantRouterShell>
                            <OperationDetailPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/resources"
                        element={
                          <TenantRouterShell>
                            <ResourceLandingPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/resources/:resourceType"
                        element={
                          <TenantRouterShell>
                            <ResourceCollectionRoute />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/resources/:resourceType/create"
                        element={
                          <TenantRouterShell>
                            <ResourceCreateRoute />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/resources/:resourceType/:id"
                        element={
                          <TenantRouterShell>
                            <ResourceDetailRoute />
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
                        path="/organization/projects"
                        element={
                          <TenantRouterShell>
                            <ProjectsPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/organization/projects/:id"
                        element={
                          <TenantRouterShell>
                            <ProjectDetailPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/organization/users"
                        element={
                          <TenantRouterShell>
                            <UsersPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/organization/users/:id"
                        element={
                          <TenantRouterShell>
                            <UserDetailPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/organization/quotas"
                        element={
                          <TenantRouterShell>
                            <QuotasPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/organization/audit"
                        element={
                          <TenantRouterShell>
                            <AuditPage />
                          </TenantRouterShell>
                        }
                      />
                      <Route
                        path="/developer/api"
                        element={
                          <TenantRouterShell>
                            <ApiCredentialsPage />
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
              </GovernanceClientProvider>
            </OperationsClientProvider>
          </ResourceClientProvider>
        </FixtureScopeProvider>
      </FixtureIdentityProvider>
    </ArafThemeProvider>
  );
}
