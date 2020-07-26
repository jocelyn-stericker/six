import { Bar, Clef, Pitch, Song, State, TiedNote } from "./state";

export interface AddNote {
  type: "ADD_NOTE";
  barIdx: number;
  startTime: [number, number];
  divisions: TiedNote;
  pitch: Pitch;
  afterBarIdx: number;
  afterTime: [number, number];
}
export function addNote(insertion: Omit<AddNote, "type">): AddNote {
  return {
    type: "ADD_NOTE",
    ...insertion,
  };
}

export interface ChangeNotePitch {
  type: "CHANGE_NOTE_PITCH";
  barIdx: number;
  startTime: [number, number];
  pitchBefore: Pitch;
  pitchAfter: Pitch;
}
export function changeNotePitch(
  appState: State,
  barIdx: number,
  startTime: [number, number],
  pitch: Pitch,
): ChangeNotePitch | null {
  let pitchBefore;
  const bar = appState.song.part.bars[barIdx];
  if (!bar) {
    return null;
  }
  for (const note of bar.notes) {
    if (
      note.startTime[0] === startTime[0] &&
      note.startTime[1] === startTime[1]
    ) {
      pitchBefore = note.pitch;
    }
  }
  if (!pitchBefore) {
    return null;
  }

  return {
    type: "CHANGE_NOTE_PITCH",
    barIdx,
    startTime,
    pitchBefore,
    pitchAfter: pitch,
  };
}

export interface RemoveNote {
  type: "REMOVE_NOTE";
  barIdx: number;
  startTime: [number, number];
  divisions: TiedNote;
  pitch: Pitch;
  beforeBarIdx: number;
  beforeTime: [number, number];
}
export function removeNote(removal: Omit<RemoveNote, "type">): RemoveNote {
  return {
    type: "REMOVE_NOTE",
    ...removal,
  };
}

export interface SetTs {
  type: "SET_TS";
  beforeBar: number;
  ts: [number, number];
  prevTs: [number, number];

  // We never change the ts of existing music.
  // Instead, we add bars when one is already filled.
  barAddCount: number;
  barKeepCount: number;
  // For undo.
  barRemoveCount: number;

  after: [number, number] | null;
}
export function setTs(
  appState: State,
  { ts, beforeBar }: { ts: [number, number]; beforeBar: number },
): SetTs {
  let barsWithoutContent = 0;
  let prevTs = appState.song.global.signatures[0].ts;
  for (let i = 0; i < beforeBar; i += 1) {
    let ts = appState.song.global.signatures[i]?.ts;
    if (ts) {
      prevTs = ts;
    }
  }

  let after = prevTs;
  while (
    appState.song.part.bars[barsWithoutContent + beforeBar]?.notes.length ===
      0 &&
    (!barsWithoutContent ||
      !appState.song.global.signatures[barsWithoutContent + beforeBar]?.ts)
  ) {
    barsWithoutContent += 1;
    after =
      appState.song.global.signatures[barsWithoutContent + beforeBar]?.ts ??
      after;
  }

  return {
    type: "SET_TS",
    beforeBar,
    ts,
    prevTs,
    barAddCount: barsWithoutContent === 0 ? 1 : 0,
    barKeepCount: barsWithoutContent,
    barRemoveCount: 0,
    after,
  };
}

export interface SetKs {
  type: "SET_KS";
  ks?: number;
  prevKs?: number;
  beforeBar: number;
}
export function setKs(
  appState: State,
  { ks, beforeBar }: { ks: number; beforeBar: number },
): SetKs {
  return {
    type: "SET_KS",
    ks,
    prevKs: appState.song.global.signatures[0].ks,
    beforeBar,
  };
}

export interface SetClef {
  type: "SET_CLEF";
  clef?: Clef;
  prevClef?: Clef;
  beforeBar: number;
}
export function setClef(
  appState: State,
  { clef, beforeBar }: { clef: Clef; beforeBar: number },
): SetClef {
  let priorClef: Clef | undefined;
  for (let i = 0; i < beforeBar; i += 1) {
    priorClef = appState.song.global.signatures[i]?.clef;
  }

  return {
    type: "SET_CLEF",
    clef: priorClef === clef ? undefined : clef,
    prevClef: appState.song.global.signatures[beforeBar]?.clef ?? undefined,
    beforeBar,
  };
}

export interface AddBar {
  type: "ADD_BAR";
  barIdx: number;
  bar: Bar;
}
export function addBar({ barIdx, bar }: { barIdx: number; bar: Bar }): AddBar {
  return {
    type: "ADD_BAR",
    barIdx,
    bar,
  };
}

export interface RemoveBar {
  type: "REMOVE_BAR";
  barIdx: number;
  bar: Bar;
}
export function removeBar({
  barIdx,
  bar,
}: {
  barIdx: number;
  bar: Bar;
}): RemoveBar {
  return {
    type: "REMOVE_BAR",
    barIdx,
    bar,
  };
}

export interface SetBarCount {
  type: "SET_BAR_COUNT";
  count: number;
  prevCount: number;
  beforeBarIdx: number;
  beforeTime: [number, number];
  afterBarIdx: number;
  afterTime: [number, number];
}
export function setBarCount(appState: State, count: number): SetBarCount {
  return {
    type: "SET_BAR_COUNT",
    count,
    prevCount: appState.song.part.bars.length,
    beforeBarIdx: appState.cursorBarIdx,
    beforeTime: appState.cursorTime,
    afterBarIdx: count - 1,
    afterTime: [0, 1],
  };
}

export interface SetTitle {
  type: "SET_TITLE";
  title: string;
  prevTitle: string;
}
export function setTitle(appState: State, title: string): SetTitle {
  return {
    type: "SET_TITLE",
    title,
    prevTitle: appState.song.global.title,
  };
}

export interface SetAuthor {
  type: "SET_AUTHOR";
  author: string;
  prevAuthor: string;
}
export function setAuthor(appState: State, author: string): SetAuthor {
  return {
    type: "SET_AUTHOR",
    author,
    prevAuthor: appState.song.global.author,
  };
}

export interface SetPickupSkip {
  type: "SET_PICKUP";
  pickupSkip: [number, number] | undefined;
  prevPickupSkip: [number, number] | undefined;
}
export function setPickupSkip(
  appState: State,
  pickupSkip: [number, number],
): SetPickupSkip {
  return {
    type: "SET_PICKUP",
    pickupSkip,
    prevPickupSkip: appState.song.global.pickupSkip,
  };
}
export function clearPickupSkip(appState: State): SetPickupSkip {
  return {
    type: "SET_PICKUP",
    pickupSkip: undefined,
    prevPickupSkip: appState.song.global.pickupSkip,
  };
}

export interface Undo {
  type: "UNDO";
}
export function undo(): Undo {
  return { type: "UNDO" };
}

export interface Redo {
  type: "REDO";
}
export function redo(): Redo {
  return { type: "REDO" };
}

export interface Reset {
  type: "RESET";
}
export function reset(): Reset {
  return { type: "RESET" };
}

export interface Load {
  type: "LOAD";
  song: Song;
}
export function load(song: Song): Load {
  return { type: "LOAD", song };
}

export interface MoveCursor {
  type: "MOVE_CURSOR";
  barIdx: number;
  time: [number, number];
}
export function moveCursor(barIdx: number, time: [number, number]): MoveCursor {
  return {
    type: "MOVE_CURSOR",
    barIdx,
    time,
  };
}

export type Invertible =
  | AddNote
  | ChangeNotePitch
  | RemoveNote
  | SetKs
  | SetTs
  | SetClef
  | AddBar
  | RemoveBar
  | SetBarCount
  | SetTitle
  | SetAuthor
  | SetPickupSkip;

export type NonInvertible = MoveCursor | Undo | Redo | Reset | Load;

export type Action = Invertible | NonInvertible;
