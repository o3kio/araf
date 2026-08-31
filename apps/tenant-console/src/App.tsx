import { BootstrapSurface } from "@araf/ui";

export function App() {
  return (
    <BootstrapSurface
      title="Araf Tenant Console"
      description="Self-service cloud console for O3K tenants. Bootstrap surface only — resource runtime, scope context and schema-driven actions arrive with later milestones."
    >
      <p>Surface: tenant (public/self-service), served by the Tenant BFF.</p>
    </BootstrapSurface>
  );
}
