import React, { memo } from "react";

/** [entity, x, y, scale] */
export type StencilMapItem = [number, number, number];
export type StencilOrStencilMap = string | Array<StencilMapItem>;
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
  stencilMeta: { [key: string]: StencilMeta };
  transform?: string;
  classNames: { [key: string]: string };
}

const StencilView = memo(function StencilView({
  id,
  stencils,
  stencilMeta,
  transform,
  classNames,
}: Props) {
  const stencil = stencils[id];
  if (!stencil) {
    return null;
  } else if (typeof stencil === "string") {
    return (
      <g
        className={classNames[id] || undefined}
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
        className={classNames[id] || undefined}
      >
        {stencil.map(([childId, x, y]) => (
          <StencilView
            key={childId}
            id={childId}
            stencils={stencils}
            stencilMeta={stencilMeta}
            classNames={classNames}
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
