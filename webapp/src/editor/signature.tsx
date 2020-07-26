import React from "react";

import { Action, Clef as ClefStr, State } from "../store";
import { Barline, Clef } from "../scene";
import css from "./signature.module.scss";

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
  beforeBar,
  children,
}: {
  appState: State;
  dispatch: (action: Action) => void;
  beforeBar: number;
  children?: any;
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
    >
      {children}
    </signature>
  );
}
