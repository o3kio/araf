import { useCallback, useEffect, useState } from "react";
import type { ArafApiError, Project, ProjectMember } from "@araf/api-client";
import { useGovernanceClient } from "../client/context";

export interface UseProjectResult {
  project: Project | undefined;
  members: ProjectMember[] | undefined;
  loading: boolean;
  error: ArafApiError | Error | undefined;
  refresh: () => void;
}

export function useProject(id: string | undefined): UseProjectResult {
  const client = useGovernanceClient();
  const [project, setProject] = useState<Project | undefined>(undefined);
  const [members, setMembers] = useState<ProjectMember[] | undefined>(undefined);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<ArafApiError | Error | undefined>(undefined);
  const [refreshToken, setRefreshToken] = useState(0);

  const refresh = useCallback(() => {
    setRefreshToken((t) => t + 1);
  }, []);

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    setLoading(true);
    setError(undefined);

    Promise.all([client.getProject(id), client.listProjectMembers(id)])
      .then(([projectResult, membersResult]) => {
        if (!cancelled) {
          setProject(projectResult);
          setMembers(membersResult);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) setError(err instanceof Error ? err : new Error(String(err)));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [client, id, refreshToken]);

  return { project, members, loading, error, refresh };
}
