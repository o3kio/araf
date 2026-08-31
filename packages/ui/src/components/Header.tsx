import { Header as CloudscapeHeader } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface HeaderProps {
  readonly children?: ReactNode;
  readonly variant?: "h1" | "h2" | "h3";
  readonly headingTagOverride?: "h1" | "h2" | "h3" | "h4" | "h5";
  readonly description?: ReactNode;
  readonly actions?: ReactNode;
  readonly counter?: ReactNode;
  readonly info?: ReactNode;
}

export function Header({ variant = "h1", ...rest }: HeaderProps) {
  return <CloudscapeHeader variant={variant} {...rest} />;
}
