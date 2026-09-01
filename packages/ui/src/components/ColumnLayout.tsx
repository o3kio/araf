import { ColumnLayout as CloudscapeColumnLayout } from "@cloudscape-design/components";
import type { ColumnLayoutProps as CloudscapeColumnLayoutProps } from "@cloudscape-design/components";

export type ColumnLayoutProps = CloudscapeColumnLayoutProps;

export function ColumnLayout(props: ColumnLayoutProps) {
  return <CloudscapeColumnLayout {...props} />;
}
