import React, { memo } from "react";

/** [entity, x, y, scale] */
export type StencilMapItem = [number, number, number];

export type StencilOrStencilMap =
  /** [className, path] */
  | [string, string]
  /* [className, children] */
  | [string, Array<StencilMapItem>];
/** [x, y, x2, y2, barIdx, timeFracNum, timeFracDen, kind] */
export type StencilMeta = [
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number,
];

export interface Props {
  id: number;
  stencils: { [key: string]: StencilOrStencilMap };
  transform?: string;
}

const StencilView = memo(function StencilView({
  id,
  stencils,
  transform,
}: Props) {
  const [className, stencil] = stencils[id];
  if (!stencil) {
    return null;
  } else if (typeof stencil === "string") {
    return (
      <g
        className={className || undefined}
        transform={transform}
        data-entity-id={id}
        dangerouslySetInnerHTML={{ __html: stencil }}
      />
    );
  } else {
    return (
      <g
        transform={transform}
        data-entity-id={id}
        className={className || undefined}
      >
        {stencil.map(([childId, x, y]) => (
          <StencilView
            key={childId}
            id={childId}
            stencils={stencils}
            transform={
              typeof x === "number" ? `translate(${x}, ${y})` : undefined
            }
          />
        ))}
      </g>
    );
  }
});

export default StencilView;
