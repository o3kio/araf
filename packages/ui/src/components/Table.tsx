import { Table as CloudscapeTable } from "@cloudscape-design/components";
import type { TableProps as CloudscapeTableProps } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export type TableColumnDefinition<T> = CloudscapeTableProps<T>["columnDefinitions"][number];

export interface TableProps<T> extends Pick<
  CloudscapeTableProps<T>,
  "items" | "columnDefinitions" | "loading" | "loadingText"
> {
  readonly trackingId?: keyof T;
  readonly header?: ReactNode;
  readonly empty?: ReactNode;
  readonly filter?: ReactNode;
  readonly preferences?: ReactNode;
  readonly selectedItems?: readonly T[];
  readonly onSelectionChange?: (items: readonly T[]) => void;
  readonly wrapLines?: boolean;
}

export function Table<T>({
  header,
  empty,
  filter,
  preferences,
  selectedItems,
  onSelectionChange,
  wrapLines,
  trackingId,
  ...rest
}: TableProps<T>) {
  const selectionType: CloudscapeTableProps<T>["selectionType"] =
    onSelectionChange != null ? "multi" : undefined;

  const trackBy: CloudscapeTableProps<T>["trackBy"] = trackingId
    ? (item: T) => String(item[trackingId])
    : undefined;

  return (
    <CloudscapeTable<T>
      {...rest}
      trackBy={trackBy}
      header={header ?? null}
      empty={empty ?? null}
      filter={filter ?? null}
      preferences={preferences ?? null}
      selectionType={selectionType}
      selectedItems={selectedItems ? [...selectedItems] : undefined}
      onSelectionChange={
        onSelectionChange
          ? ({ detail }) => {
              onSelectionChange(detail.selectedItems);
            }
          : undefined
      }
      wrapLines={wrapLines}
      stripedRows
      stickyHeader
    />
  );
}
