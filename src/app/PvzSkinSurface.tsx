import type { CSSProperties, ReactNode } from "react";

import { pvzUiCssVariables } from "../assets/pvz-ui";

interface PvzSkinSurfaceProps {
  children: ReactNode;
}

export function PvzSkinSurface({ children }: PvzSkinSurfaceProps) {
  return (
    <div
      className="pvz-skin-root"
      style={pvzUiCssVariables as CSSProperties}
    >
      {children}
    </div>
  );
}
