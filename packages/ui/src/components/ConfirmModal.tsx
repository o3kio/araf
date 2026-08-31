import { Button, Modal, SpaceBetween } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface ConfirmModalProps {
  readonly open: boolean;
  readonly title: string;
  readonly children: ReactNode;
  readonly confirmLabel?: string;
  readonly cancelLabel?: string;
  readonly closeAriaLabel?: string;
  readonly onConfirm: () => void;
  readonly onCancel: () => void;
  readonly loading?: boolean;
}

export function ConfirmModal({
  open,
  title,
  children,
  confirmLabel = "Confirm",
  cancelLabel = "Cancel",
  closeAriaLabel = "Close dialog",
  onConfirm,
  onCancel,
  loading = false,
}: ConfirmModalProps) {
  return (
    <Modal visible={open} onDismiss={onCancel} header={title} closeAriaLabel={closeAriaLabel}>
      <SpaceBetween size="m" direction="vertical">
        {children}
        <SpaceBetween size="xs" direction="horizontal">
          <Button variant="link" onClick={onCancel} disabled={loading}>
            {cancelLabel}
          </Button>
          <Button variant="primary" onClick={onConfirm} loading={loading}>
            {confirmLabel}
          </Button>
        </SpaceBetween>
      </SpaceBetween>
    </Modal>
  );
}
