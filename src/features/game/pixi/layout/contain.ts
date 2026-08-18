export const LOGICAL_SCENE_WIDTH = 1400;
export const LOGICAL_SCENE_HEIGHT = 600;

export interface ContainLayout {
  scale: number;
  displayWidth: number;
  displayHeight: number;
  offsetX: number;
  offsetY: number;
}

export function calculateContainLayout(
  containerWidth: number,
  containerHeight: number,
  logicalWidth = LOGICAL_SCENE_WIDTH,
  logicalHeight = LOGICAL_SCENE_HEIGHT,
): ContainLayout {
  if (
    containerWidth <= 0 ||
    containerHeight <= 0 ||
    logicalWidth <= 0 ||
    logicalHeight <= 0
  ) {
    return { scale: 0, displayWidth: 0, displayHeight: 0, offsetX: 0, offsetY: 0 };
  }

  const scale = Math.min(containerWidth / logicalWidth, containerHeight / logicalHeight);
  const displayWidth = logicalWidth * scale;
  const displayHeight = logicalHeight * scale;

  return {
    scale,
    displayWidth,
    displayHeight,
    offsetX: (containerWidth - displayWidth) / 2,
    offsetY: (containerHeight - displayHeight) / 2,
  };
}
