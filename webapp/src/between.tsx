import React from "react";

import {
  Action,
  State,
  Clef as ClefStr,
  addBar,
  removeBar,
  setKs,
  setTs,
  setClef,
} from "./store";
import { Barline, Clef } from "./sheet/reconciler";

const BetweenBarPopover = React.lazy(() => import("./between_bar_popover"));

function clefStrToNum(clef: ClefStr): Clef {
  if (clef === "g") {
    return Clef.G;
  }
  if (clef === "f") {
    return Clef.F;
  }
  if (clef === "percussion") {
    return Clef.Percussion;
  }

  throw new Error("Unexpected clef");
}

export default function Between({
  appState,
  dispatch,
  beforeBar,
}: {
  appState: State;
  dispatch: (action: Action) => void;
  beforeBar: number;
}) {
  const bar = appState.song.part.bars[beforeBar - 1];
  const between = appState.song.global.between[beforeBar];
  let barline: Barline | undefined;
  if (bar) {
    barline = bar.barline === "normal" ? Barline.Normal : Barline.Final;
  }
  const clef = between?.clef;

  return (
    <between
      barline={barline}
      clef={clef && clefStrToNum(clef)}
      tsNum={between?.ts?.[0]}
      tsDen={between?.ts?.[1]}
      ks={between?.ks}
      className="between-bars"
      html={({ width, height }) => (
        <React.Suspense fallback={null}>
          <BetweenBarPopover
            ts={between?.ts}
            setClef={clef => {
              dispatch(setClef(appState, clef));
            }}
            setKs={ks => {
              dispatch(setKs(appState, ks));
            }}
            setTs={([num, den]) => {
              dispatch(setTs(appState, { beforeBar, ts: [num, den] }));
            }}
            onInsertBarRight={() => {
              dispatch(
                addBar({
                  barIdx: beforeBar,
                  bar: {
                    barline: "normal",
                    notes: [],
                  },
                }),
              );
            }}
            onRemoveBarRight={
              (appState.song.part.bars.length > 1 &&
                appState.song.part.bars[beforeBar] &&
                (() => {
                  dispatch(
                    removeBar({
                      barIdx: beforeBar,
                      bar: appState.song.part.bars[beforeBar],
                    }),
                  );
                })) ||
              null
            }
          >
            <div
              className="between-edit"
              style={{
                width,
                height,
                cursor: "pointer",
              }}
            />
          </BetweenBarPopover>
        </React.Suspense>
      )}
    />
  );
}
