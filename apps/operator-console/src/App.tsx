import { ArafThemeProvider } from "@araf/ui";
import { FixtureIdentityProvider, OperatorShell, type OperatorNavigationItem } from "@araf/shell";
import {
  ResourceClientProvider,
  ResourceLandingPage,
  ResourceCollectionPage,
  ResourceDetailPage,
} from "@araf/resources";
import { createArafClient } from "@araf/api-client";
import { useState, type ReactNode } from "react";
import { BrowserRouter, Routes, Route, Navigate, useLocation, Link, useParams } from "react-router";

const operatorBffUrl =
  (import.meta.env.VITE_OPERATOR_BFF_URL as string | undefined) ?? "http://127.0.0.1:8081";
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
      { id: "projects", type: "link", text: "Projects", href: "/customers/projects" },
    ],
  },
  {
    id: "services",
    type: "section",
    text: "Services",
    items: [
      { id: "catalog", type: "link", text: "Catalog", href: "/services/catalog" },
      { id: "installed", type: "link", text: "Installed Services", href: "/services/installed" },
      { id: "resources", type: "link", text: "Resources", href: "/resources" },
    ],
  },
  {
    id: "infrastructure",
    type: "section",
    text: "Infrastructure",
    items: [
      {
        id: "compute-providers",
        type: "link",
        text: "Compute Providers",
        href: "/infrastructure/compute",
      },
      {
        id: "network-providers",
        type: "link",
        text: "Network Providers",
        href: "/infrastructure/network",
      },
      {
        id: "storage-providers",
        type: "link",
        text: "Storage Providers",
        href: "/infrastructure/storage",
      },
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
      { id: "iam", type: "link", text: "IAM", href: "/governance/iam" },
      { id: "quotas", type: "link", text: "Quotas", href: "/governance/quotas" },
      { id: "metering", type: "link", text: "Metering", href: "/governance/metering" },
      { id: "audit", type: "link", text: "Audit", href: "/governance/audit" },
    ],
  },
];

function OverviewPage() {
  return (
    <section>
      <h1>Platform overview</h1>
      <p>Operator view of the O3K platform.</p>
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

export function App() {
  const [density] = useState<"comfortable" | "compact">("compact");

  return (
    <ArafThemeProvider density={density}>
      <FixtureIdentityProvider
        initialIdentity={{ userId: "fixture-operator", userName: "Platform Operator" }}
      >
        <ResourceClientProvider client={arafClient}>
          <BrowserRouter>
            <Routes>
              <Route path="/" element={<Navigate to="/platform/overview" replace />} />
              <Route
                path="/platform/overview"
                element={
                  <OperatorRouterShell>
                    <OverviewPage />
                  </OperatorRouterShell>
                }
              />
              <Route
                path="/platform/*"
                element={
                  <OperatorRouterShell>
                    <PlaceholderPage title="Platform" />
                  </OperatorRouterShell>
                }
              />
              <Route
                path="/customers/*"
                element={
                  <OperatorRouterShell>
                    <PlaceholderPage title="Customers" />
                  </OperatorRouterShell>
                }
              />
              <Route
                path="/services/catalog"
                element={
                  <OperatorRouterShell>
                    <PlaceholderPage title="Catalog" />
                  </OperatorRouterShell>
                }
              />
              <Route
                path="/services/installed"
                element={
                  <OperatorRouterShell>
                    <PlaceholderPage title="Installed Services" />
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
                    <PlaceholderPage title="Infrastructure" />
                  </OperatorRouterShell>
                }
              />
              <Route
                path="/operations"
                element={
                  <OperatorRouterShell>
                    <PlaceholderPage title="Operations" />
                  </OperatorRouterShell>
                }
              />
              <Route
                path="/governance/*"
                element={
                  <OperatorRouterShell>
                    <PlaceholderPage title="Governance" />
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
        </ResourceClientProvider>
      </FixtureIdentityProvider>
    </ArafThemeProvider>
  );
}
