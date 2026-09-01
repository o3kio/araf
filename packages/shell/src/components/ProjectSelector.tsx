import type { ProjectOption } from "../types";

export interface ProjectSelectorProps {
  id?: string;
  label?: string;
  projects: ProjectOption[];
  selectedProjectId?: string;
  onSelectProject: (projectId: string) => void;
  disabled?: boolean;
  placeholder?: string;
}

/**
 * Accessible project selector.
 */
export function ProjectSelector({
  id = "project-selector",
  label = "Project",
  projects,
  selectedProjectId,
  onSelectProject,
  disabled = false,
  placeholder = "Select a project",
}: ProjectSelectorProps) {
  return (
    <div className="araf-project-selector">
      <label htmlFor={id} className="araf-project-selector__label">
        {label}
      </label>
      <select
        id={id}
        name="project"
        value={selectedProjectId ?? ""}
        onChange={(event) => {
          onSelectProject(event.target.value);
        }}
        disabled={disabled || projects.length === 0}
        aria-label={label}
      >
        <option value="" disabled>
          {placeholder}
        </option>
        {projects.map((project) => (
          <option key={project.id} value={project.id}>
            {project.name}
          </option>
        ))}
      </select>
    </div>
  );
}
