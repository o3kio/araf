import { Container, Header, SpaceBetween } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface FormSectionProps {
  readonly title: string;
  readonly description?: string;
  readonly children: ReactNode;
}

export function FormSection({ title, description, children }: FormSectionProps) {
  return (
    <Container
      header={
        <Header variant="h2" description={description}>
          {title}
        </Header>
      }
    >
      <SpaceBetween size="l" direction="vertical">
        {children}
      </SpaceBetween>
    </Container>
  );
}
