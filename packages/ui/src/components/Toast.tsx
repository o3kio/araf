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

function toFlashbarItem(item: ToastItem): FlashbarProps["items"][number] {
  return {
    id: item.id,
    type: typeMap[item.type],
    content: item.message,
    header: item.header,
    dismissible: true,
    dismissLabel: "Dismiss notification",
    buttonText: item.action?.label,
    onButtonClick: item.action?.onPress,
  };
}

export interface ToastProps {
  readonly items: readonly ToastItem[];
  readonly onDismiss?: (id: string) => void;
}

export function Toast({ items, onDismiss }: ToastProps) {
  const flashbarItems = items.map((item) => ({
    ...toFlashbarItem(item),
    onDismiss: onDismiss
      ? () => {
          onDismiss(item.id);
        }
      : undefined,
  }));

  return (
    <section aria-label="Notifications">
      <Flashbar items={flashbarItems} />
    </section>
  );
}
