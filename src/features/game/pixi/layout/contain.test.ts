import { describe, expect, it } from "vitest";

import { calculateContainLayout } from "./contain";

describe("fixed-world contain mapping", () => {
  it("fits a wide host without changing logical aspect ratio", () => {
    expect(calculateContainLayout(700, 300)).toEqual({
      scale: 0.5,
      displayWidth: 700,
      displayHeight: 300,
      offsetX: 0,
      offsetY: 0,
    });
  });

  it("letterboxes narrower and taller hosts", () => {
    const layout = calculateContainLayout(700, 400);
    expect(layout.scale).toBe(0.5);
    expect(layout.offsetX).toBe(0);
    expect(layout.offsetY).toBe(50);
  });

  it("returns an inert layout until the host is measurable", () => {
    expect(calculateContainLayout(0, 300).scale).toBe(0);
  });
});
