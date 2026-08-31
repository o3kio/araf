import { Table as CloudscapeTable } from "@cloudscape-design/components";
import type { TableProps as CloudscapeTableProps } from "@cloudscape-design/components";
import type { ReactNode } from "react";

export interface TableColumnDefinition<T> {
  readonly id: string;
  readonly header: string;
  readonly cell: (item: T) => ReactNode;
  readonly width?: string | number;
  readonly isRowHeader?: boolean;
}

export interface TableAriaLabels {
  readonly selectionGroupLabel?: string;
  readonly allItemsSelectionLabel?: string;
  readonly itemSelectionLabel?: (item: unknown) => string;
  readonly tableLabel?: string;
}

export interface TableProps<T> {
  readonly items: readonly T[];
  readonly columnDefinitions: readonly TableColumnDefinition<T>[];
  readonly trackingId?: keyof T;
  readonly header?: ReactNode;
  readonly empty?: ReactNode;
  readonly filter?: ReactNode;
  readonly preferences?: ReactNode;
  readonly selectedItems?: readonly T[];
  readonly onSelectionChange?: (items: readonly T[]) => void;
  readonly wrapLines?: boolean;
  readonly loading?: boolean;
  readonly loadingText?: string;
  readonly ariaLabels?: TableAriaLabels;
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
  ariaLabels,
  ...rest
}: TableProps<T>) {
  const selectionType: CloudscapeTableProps<T>["selectionType"] =
    onSelectionChange != null ? "multi" : undefined;

  const trackBy: CloudscapeTableProps<T>["trackBy"] = trackingId
    ? (item: T) => String(item[trackingId])
    : undefined;

  const cloudscapeAriaLabels: CloudscapeTableProps<T>["ariaLabels"] = (() => {
    if (!ariaLabels) return undefined;
    const allItemsLabel = ariaLabels.allItemsSelectionLabel;
    return {
      selectionGroupLabel: ariaLabels.selectionGroupLabel,
      allItemsSelectionLabel: allItemsLabel ? () => allItemsLabel : undefined,
      itemSelectionLabel: (_state: unknown, row: T) => ariaLabels.itemSelectionLabel?.(row) ?? "",
      tableLabel: ariaLabels.tableLabel,
    };
  })();

  const cloudscapeColumns: CloudscapeTableProps<T>["columnDefinitions"] =
    rest.columnDefinitions.map((col) => ({
      id: col.id,
      header: col.header,
      cell: col.cell,
      width: col.width,
      isRowHeader: col.isRowHeader,
    }));

  return (
    <CloudscapeTable<T>
      {...rest}
      columnDefinitions={cloudscapeColumns}
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
      ariaLabels={cloudscapeAriaLabels}
      wrapLines={wrapLines}
      stripedRows
      stickyHeader
    />
  );
}
