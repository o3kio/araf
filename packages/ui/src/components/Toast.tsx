import { Flashbar } from "@cloudscape-design/components";
import type { FlashbarProps } from "@cloudscape-design/components";

export type ArafToastType = "success" | "error" | "warning" | "info";

export interface ToastAction {
  readonly label: string;
  readonly onPress: () => void;
}

export interface ToastItem {
  readonly id: string;
  readonly type: ArafToastType;
  readonly message: string;
  readonly header?: string;
  readonly action?: ToastAction;
}

const typeMap: Record<ArafToastType, FlashbarProps["items"][number]["type"]> = {
  success: "success",
  error: "error",
  warning: "warning",
  info: "info",
};

function toFlashbarItem(
  item: ToastItem,
  onDismiss: ToastProps["onDismiss"],
  dismissLabel: string,
): FlashbarProps["items"][number] {
  return {
    id: item.id,
    type: typeMap[item.type],
    content: item.message,
    header: item.header,
    dismissible: onDismiss != null,
    dismissLabel,
    buttonText: item.action?.label,
    onButtonClick: item.action?.onPress,
    onDismiss: onDismiss
      ? () => {
          onDismiss(item.id);
        }
      : undefined,
  };
}

export interface ToastProps {
  readonly items: readonly ToastItem[];
  readonly onDismiss?: (id: string) => void;
  readonly notificationsLabel?: string;
  readonly dismissLabel?: string;
}

export function Toast({
  items,
  onDismiss,
  notificationsLabel = "Notifications",
  dismissLabel = "Dismiss notification",
}: ToastProps) {
  const flashbarItems = items.map((item) => toFlashbarItem(item, onDismiss, dismissLabel));

  return (
    <section aria-label={notificationsLabel}>
      <Flashbar items={flashbarItems} />
    </section>
  );
}
