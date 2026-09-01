import { Container as CloudscapeContainer } from "@cloudscape-design/components";
import type { ContainerProps as CloudscapeContainerProps } from "@cloudscape-design/components";

export type ContainerProps = CloudscapeContainerProps;

export function Container(props: ContainerProps) {
  return <CloudscapeContainer {...props} />;
}
