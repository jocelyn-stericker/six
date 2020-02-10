import React, { useLayoutEffect, useState, useEffect, useRef } from "react";
import { render, newRender } from "./reconciler";

export type Render = import("./reconciler").Render;
export { NoteValue, Barline } from "./reconciler";
export const TYPE_RNC = 0;
export const TYPE_BETWEEN_BARS = 1;

interface Props {
  children: any;
  onMouseDown?: (
    time: null | [number, number, number],
    ev: React.MouseEvent
  ) => void;
  onMouseUp?: (
    time: null | [number, number, number],
    ev: React.MouseEvent
  ) => void;
  onClick?: (
    time: null | [number, number, number],
    ev: React.MouseEvent
  ) => void;
  onMouseMove?: (ev: React.MouseEvent) => void;
  onHoverTimeChanged: (time: [number, number, number] | null) => void;
}

/** [entity, x, y, scale] */
type StencilMapItem = [number, number, number, number];
type StencilOrStencilMap = string | Array<StencilMapItem>;
/** [x, y, x2, y2, barIdx, timeFracNum, timeFracDen, kind] */
type StencilMeta = [
  number,
  number,
  number,
  number,
  number,
  number,
  number,
  number
];

function StencilView({
  id,
  stencils,
  stencilMeta,
  transform,
  classNames
}: {
  id: number;
  stencils: { [key: string]: StencilOrStencilMap };
  stencilMeta: { [key: string]: StencilMeta };
  transform?: string;
  classNames: { [key: string]: string };
}) {
  const stencil = stencils[id];
  if (typeof stencil === "string") {
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
        {stencil.map(([childId, x, y, scale]) => (
          <StencilView
            key={childId}
            id={childId}
            stencils={stencils}
            stencilMeta={stencilMeta}
            classNames={classNames}
            transform={
              typeof x === "number"
                ? `translate(${x}, ${y}) scale(${scale})`
                : undefined
            }
          />
        ))}
      </g>
    );
  }
}

export default function SheetMusicView(props: Props) {
  // create/destroy Rust container
  const [container] = useState(newRender);
  useEffect(() => {
    return () => {
      container.free();
    };
  }, [container]);

  // render loop
  const [stencils, setStencils] = useState<{
    [key: number]: StencilOrStencilMap;
  } | null>(null);
  const [stencilMeta, setStencilMeta] = useState<{
    [key: number]: StencilMeta;
  } | null>(null);
  const [root, setRoot] = useState<number | null>(null);
  const [hoverTime, setHoverTime] = useState<[number, number, number] | null>(
    null
  );

  useLayoutEffect(() => {
    console.time("render svg");
    render(props.children, container);
    container.exec();
    let stencilPairs = container.stencils().split("\n");
    let stencilMapPairs = container.stencil_maps().split("\n");
    let stencilMetaPairs = container.get_stencil_bboxes().split("\n");

    let stencils: { [key: number]: StencilOrStencilMap } = {};
    for (let i = 0; i < stencilPairs.length; i += 2) {
      stencils[stencilPairs[i] as any] = stencilPairs[i + 1];
    }

    for (let i = 0; i < stencilMapPairs.length; i += 2) {
      stencils[stencilMapPairs[i] as any] = JSON.parse(stencilMapPairs[i + 1]);
    }

    let stencilMeta: { [key: number]: StencilMeta } = {};
    for (let i = 0; i < stencilMetaPairs.length; i += 2) {
      stencilMeta[stencilMetaPairs[i] as any] = JSON.parse(
        stencilMetaPairs[i + 1]
      );
    }

    console.timeEnd("render svg");

    setStencils(stencils);
    setStencilMeta(stencilMeta);
    setRoot(container.get_root_id() || null);
  }, [container, props.children]);

  const svg = useRef<SVGSVGElement>(null);

  const bound = svg.current && svg.current.getBoundingClientRect();

  function makeMouseHandler(
    fn?: (time: null | [number, number, number], ev: React.MouseEvent) => void
  ) {
    return (ev: React.MouseEvent) => {
      if (!stencilMeta || !fn) {
        return;
      }

      fn(hoverTime, ev);
    };
  }

  return (
    <>
      <svg
        viewBox="0 0 215.9 279.4"
        width="100%"
        ref={svg}
        onMouseDownCapture={makeMouseHandler(props.onMouseDown)}
        onMouseUpCapture={makeMouseHandler(props.onMouseUp)}
        onClick={makeMouseHandler(props.onClick)}
        onMouseMoveCapture={ev => {
          if (!svg || !svg.current || !stencilMeta) {
            return;
          }
          const ctm = svg.current.getScreenCTM();
          if (!ctm) {
            return;
          }
          let pt = svg.current.createSVGPoint();
          pt.x = ev.clientX;
          pt.y = ev.clientY;
          pt = pt.matrixTransform(ctm.inverse());
          pt.y = -pt.y;

          const time = container.get_time_for_cursor(pt.x, pt.y);

          if (
            Boolean(hoverTime) !== Boolean(time) ||
            (time &&
              hoverTime &&
              (time[0] !== hoverTime[0] ||
                time[1] !== hoverTime[1] ||
                time[2] !== hoverTime[2]))
          ) {
            setHoverTime(time ? [time[0], time[1], time[2]] : null);
            props.onHoverTimeChanged(time ? [time[0], time[1], time[2]] : null);
          }

          if (props.onMouseMove) {
            props.onMouseMove(ev);
          }
        }}
      >
        <g transform="scale(1, -1)">
          {stencilMeta &&
            Object.entries(container.boundingClassNames).map(
              ([id, className]) => {
                const meta = stencilMeta[id as any];
                if (!meta || !className) {
                  return null;
                }
                return (
                  <rect
                    key={id}
                    x={meta[0]}
                    y={meta[1]}
                    width={meta[2] - meta[0]}
                    height={meta[3] - meta[1]}
                    className={className}
                  />
                );
              }
            )}
          {root && stencils && stencils[root] && stencilMeta && (
            <g style={{ pointerEvents: "none" }}>
              <StencilView
                id={root}
                stencils={stencils}
                stencilMeta={stencilMeta}
                classNames={container.classNames}
              />
            </g>
          )}
        </g>
      </svg>
      {stencilMeta &&
        Object.entries(container.html).map(([id, html]) => {
          const meta = stencilMeta[id as any];
          if (!meta || !html || !svg.current || !bound) {
            return null;
          }

          const ctm = svg.current.getScreenCTM();
          if (!ctm) {
            return;
          }

          let pt2 = svg.current.createSVGPoint();
          pt2.x = meta[0];
          pt2.y = -meta[1];
          pt2 = pt2.matrixTransform(ctm);

          let pt3 = svg.current.createSVGPoint();
          pt3.x = meta[2];
          pt3.y = -meta[3];
          pt3 = pt3.matrixTransform(ctm);

          const width = pt3.x - pt2.x;
          const height = pt2.y - pt3.y;
          return (
            <div
              key={id}
              style={{
                position: "absolute",
                left: pt2.x - bound.left,
                top: pt3.y - bound.top,
                width,
                height
              }}
            >
              <div style={{ position: "relative" }}>
                {html({ width, height })}
              </div>
            </div>
          );
        })}
    </>
  );
}
