/**
 * Phase 5A uses zero-based lawn coordinates: rows 0..2 and columns 0..6.
 * These logical coordinates never depend on the browser or window dimensions.
 */
export const LAWN_GRID = {
  rows: 3,
  columns: 7,
  originX: 245,
  originY: 145,
  cellWidth: 105,
  cellHeight: 125,
} as const;

export interface LawnCell {
  row: number;
  column: number;
}

export interface Point {
  x: number;
  y: number;
}

export function isLawnCell(cell: LawnCell): boolean {
  return (
    Number.isInteger(cell.row) &&
    Number.isInteger(cell.column) &&
    cell.row >= 0 &&
    cell.row < LAWN_GRID.rows &&
    cell.column >= 0 &&
    cell.column < LAWN_GRID.columns
  );
}

export function getCellCenter(row: number, column: number): Point {
  const cell = { row, column };
  if (!isLawnCell(cell)) {
    throw new RangeError(`Lawn cell out of bounds: row=${row}, column=${column}`);
  }

  return {
    x: LAWN_GRID.originX + (column + 0.5) * LAWN_GRID.cellWidth,
    y: LAWN_GRID.originY + (row + 0.5) * LAWN_GRID.cellHeight,
  };
}
