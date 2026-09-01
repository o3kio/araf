import { useScope } from "../scope/context";

export interface ScopeDisplayProps {
  label?: string;
}

/**
 * Read-only display of the currently selected scope.
 *
 * Helps satisfy the M2 UX invariant: a user must not perform a cloud action
 * while uncertain which project and region it targets.
 */
export function ScopeDisplay({ label = "Scope" }: ScopeDisplayProps) {
  const { scope } = useScope();

  const project = scope.projectName ?? scope.projectId ?? "No project";
  const region = scope.regionName ?? scope.regionId ?? "Global";

  return (
    <div className="araf-scope-display" aria-label={label}>
      <span className="araf-scope-display__project" data-testid="scope-project">
        {project}
      </span>
      <span className="araf-scope-display__separator" aria-hidden="true">
        /
      </span>
      <span className="araf-scope-display__region" data-testid="scope-region">
        {region}
      </span>
    </div>
  );
}
