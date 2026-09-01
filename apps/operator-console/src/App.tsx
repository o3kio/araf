import { ArafThemeProvider } from "@araf/ui";
import { FixtureIdentityProvider, OperatorShell, type OperatorNavigationItem } from "@araf/shell";
import {
  ResourceClientProvider,
  ResourceLandingPage,
  ResourceCollectionPage,
  ResourceDetailPage,
  ResourceCreatePage,
} from "@araf/resources";
import { OperationsClientProvider, OperationDetailPage } from "@araf/operations";
import {
  OperatorPlatformClientProvider,
  PlatformOverviewPage,
  RegionsPage,
  RegionDetailPage,
  ProviderHealthPage,
  CapacityPage,
  AccountsPage,
  AccountProjectsPage,
  OperatorOperationsPage,
  OperatorAuditPage,
  InstalledServicesPage,
} from "@araf/operator-platform";
import { createArafClient } from "@araf/api-client";
import { useState, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate, useLocation, Link, useParams } from "react-router";

// Default to same-origin: in dev the Vite server proxies /api to the BFF,
// in production nginx proxies it (see deploy/nginx-default.conf.template).
// VITE_OPERATOR_BFF_URL remains as an explicit override (used by e2e tests).
const operatorBffUrl =
  (import.meta.env.VITE_OPERATOR_BFF_URL as string | undefined) ?? window.location.origin;
const arafClient = createArafClient(operatorBffUrl);

const navigationItems: OperatorNavigationItem[] = [
  {
    id: "platform",
    type: "section",
    text: "Platform",
    items: [
      { id: "overview", type: "link", text: "Overview", href: "/platform/overview" },
      { id: "regions", type: "link", text: "Regions", href: "/platform/regions" },
      { id: "health", type: "link", text: "Health", href: "/platform/health" },
      { id: "capacity", type: "link", text: "Capacity", href: "/platform/capacity" },
    ],
  },
  {
    id: "customers",
    type: "section",
    text: "Customers",
    items: [
      { id: "accounts", type: "link", text: "Accounts", href: "/customers/accounts" },
    ],
  },
  {
    id: "services",
    type: "section",
    text: "Services",
    items: [
      { id: "installed", type: "link", text: "Installed Services", href: "/services/installed" },
      { id: "resources", type: "link", text: "Resources", href: "/resources" },
    ],
  },
  {
    id: "operations",
    type: "section",
    text: "Operations",
    items: [{ id: "operations-list", type: "link", text: "Operations", href: "/operations" }],
  },
  {
    id: "governance",
    type: "section",
    text: "Governance",
    items: [
      { id: "audit", type: "link", text: "Audit", href: "/governance/audit" },
    ],
  },
];

function UnavailablePage({ title }: { title: string }) {
  return (
    <section>
      <h1>{title}</h1>
      <p>This capability is not available in the current deployment.</p>
    </section>
  );
}

function NotFound() {
  return (
    <section>
      <h1>Not found</h1>
      <p>The requested operator page does not exist.</p>
      <Link to="/platform/overview">Go to overview</Link>
    </section>
  );
}

function OperatorRouterShell({ children }: { children: ReactNode }) {
  const location = useLocation();
  return (
    <OperatorShell navigationItems={navigationItems} activeHref={location.pathname}>
      {children}
    </OperatorShell>
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

function RegionDetailRoute() {
  return <RegionDetailPage />;
}

function AccountProjectsRoute() {
  return <AccountProjectsPage />;
}

export function App() {
  const [density] = useState<"comfortable" | "compact">("compact");

  return (
    <ArafThemeProvider density={density}>
      <FixtureIdentityProvider
        initialIdentity={{ userId: "fixture-operator", userName: "Platform Operator" }}
      >
        <ResourceClientProvider client={arafClient}>
          <OperationsClientProvider client={arafClient}>
            <OperatorPlatformClientProvider client={arafClient}>
              <BrowserRouter>
                <Routes>
                  <Route path="/" element={<Navigate to="/platform/overview" replace />} />
                  <Route
                    path="/platform/overview"
                    element={
                      <OperatorRouterShell>
                        <PlatformOverviewPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/platform/regions"
                    element={
                      <OperatorRouterShell>
                        <RegionsPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/platform/regions/:id"
                    element={
                      <OperatorRouterShell>
                        <RegionDetailRoute />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/platform/health"
                    element={
                      <OperatorRouterShell>
                        <ProviderHealthPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/platform/capacity"
                    element={
                      <OperatorRouterShell>
                        <CapacityPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/customers/accounts"
                    element={
                      <OperatorRouterShell>
                        <AccountsPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/customers/accounts/:id/projects"
                    element={
                      <OperatorRouterShell>
                        <AccountProjectsRoute />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/customers/projects"
                    element={
                      <OperatorRouterShell>
                        <UnavailablePage title="Projects" />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/services/installed"
                    element={
                      <OperatorRouterShell>
                        <InstalledServicesPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/resources"
                    element={
                      <OperatorRouterShell>
                        <ResourceLandingPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/resources/:resourceType"
                    element={
                      <OperatorRouterShell>
                        <ResourceCollectionRoute />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/resources/:resourceType/create"
                    element={
                      <OperatorRouterShell>
                        <ResourceCreateRoute />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/resources/:resourceType/:id"
                    element={
                      <OperatorRouterShell>
                        <ResourceDetailRoute />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/infrastructure/*"
                    element={
                      <OperatorRouterShell>
                        <UnavailablePage title="Infrastructure" />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/operations"
                    element={
                      <OperatorRouterShell>
                        <OperatorOperationsPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/operations/:id"
                    element={
                      <OperatorRouterShell>
                        <OperationDetailPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/governance/audit"
                    element={
                      <OperatorRouterShell>
                        <OperatorAuditPage />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="/governance/*"
                    element={
                      <OperatorRouterShell>
                        <UnavailablePage title="Governance" />
                      </OperatorRouterShell>
                    }
                  />
                  <Route
                    path="*"
                    element={
                      <OperatorRouterShell>
                        <NotFound />
                      </OperatorRouterShell>
                    }
                  />
                </Routes>
              </BrowserRouter>
            </OperatorPlatformClientProvider>
          </OperationsClientProvider>
        </ResourceClientProvider>
      </FixtureIdentityProvider>
    </ArafThemeProvider>
  );
}
