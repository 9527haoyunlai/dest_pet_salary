import { describe, expect, it } from "vitest";

import { getCellCenter, isLawnCell, LAWN_GRID } from "./lawnGrid";

describe("3 by 7 lawn coordinate system", () => {
  it("maps zero-based cells to deterministic centers", () => {
    expect(getCellCenter(0, 0)).toEqual({
      x: LAWN_GRID.originX + LAWN_GRID.cellWidth / 2,
      y: LAWN_GRID.originY + LAWN_GRID.cellHeight / 2,
    });
    expect(getCellCenter(2, 6)).toEqual({
      x: LAWN_GRID.originX + LAWN_GRID.cellWidth * 6.5,
      y: LAWN_GRID.originY + LAWN_GRID.cellHeight * 2.5,
    });
  });

  it("accepts only rows 0..2 and columns 0..6", () => {
    expect(isLawnCell({ row: 0, column: 0 })).toBe(true);
    expect(isLawnCell({ row: 2, column: 6 })).toBe(true);
    expect(isLawnCell({ row: -1, column: 0 })).toBe(false);
    expect(isLawnCell({ row: 3, column: 0 })).toBe(false);
    expect(isLawnCell({ row: 0, column: 7 })).toBe(false);
    expect(() => getCellCenter(3, 1)).toThrow(RangeError);
  });
});
