import { BootstrapSurface } from "@araf/ui";

export function App() {
  return (
    <BootstrapSurface
      title="Araf Operator Console"
      description="Management surface for O3K platform operators. Separate application, BFF and deployment boundary from the Tenant Console (ADR 0001)."
    >
      <p>Surface: operator (management network), served by the Operator BFF.</p>
    </BootstrapSurface>
  );
}
