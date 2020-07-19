import React from "react";

import {
  Action,
  addBar,
  Clef as ClefStr,
  removeBar,
  setClef,
  setKs,
  setTs,
  State,
} from "../store";
import { Barline, Clef } from "../scene";
import css from "./signature.module.scss";

const SignaturePopover = React.lazy(() => import("./signature_popover"));

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

export default function Signature({
  appState,
  dispatch,
  beforeBar,
}: {
  appState: State;
  dispatch: (action: Action) => void;
  beforeBar: number;
}) {
  const bar = appState.song.part.bars[beforeBar - 1];
  const signature = appState.song.global.signatures[beforeBar];
  let barline: Barline | undefined;
  if (bar) {
    barline = bar.barline === "normal" ? Barline.Normal : Barline.Final;
  }
  const clef = signature?.clef;

  return (
    <signature
      barline={barline}
      clef={clef && clefStrToNum(clef)}
      tsNum={signature?.ts?.[0]}
      tsDen={signature?.ts?.[1]}
      ks={signature?.ks}
      className={css.signature}
      html={({ width, height }) => (
        <React.Suspense fallback={null}>
          <SignaturePopover
            ts={signature?.ts}
            setClef={clef => {
              dispatch(setClef(appState, { clef, beforeBar }));
            }}
            setKs={ks => {
              dispatch(setKs(appState, { ks, beforeBar }));
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
              className={css.signatureEdit}
              style={{
                width,
                height,
                cursor: "pointer",
              }}
            />
          </SignaturePopover>
        </React.Suspense>
      )}
    />
  );
}
