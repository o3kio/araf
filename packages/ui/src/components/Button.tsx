import { Button as CloudscapeButton } from "@cloudscape-design/components";
import type { ButtonProps as CloudscapeButtonProps } from "@cloudscape-design/components";

export type ButtonProps = CloudscapeButtonProps;

export function Button(props: ButtonProps) {
  return <CloudscapeButton {...props} />;
}
