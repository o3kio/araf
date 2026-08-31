import { FormField as CloudscapeFormField } from "@cloudscape-design/components";
import type { FormFieldProps as CloudscapeFormFieldProps } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface FormFieldProps extends Pick<
  CloudscapeFormFieldProps,
  "label" | "description" | "errorText" | "stretch" | "constraintText" | "secondaryControl"
> {
  readonly children: ReactNode;
  readonly id?: string;
}

export function FormField({ id, children, ...rest }: FormFieldProps) {
  return (
    <CloudscapeFormField {...rest} controlId={id}>
      {children}
    </CloudscapeFormField>
  );
}
