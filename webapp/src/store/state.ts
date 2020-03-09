import { Invertible } from "./actions";

export type Clef = "g" | "f" | "percussion";

export interface Between {
  ts: [number, number];
  ks: number;
  clef: Clef;
}

export interface Global {
  title: string;
  author: string;
  between: [Between, ...Array<Partial<Between> | undefined>];
}

export type TiedNote = Array<{
  noteValue: number;
  dots: number;
  startTime: [number, number];
}>;

export interface Note {
  startTime: [number, number];
  divisions: TiedNote;
}

export interface Bar {
  barline: "normal" | "final";
  notes: Array<Note>;
}

export interface Part {
  bars: Array<Bar>;
}

export interface Song {
  v: "0.1.0";
  global: Global;
  part: Part;
}

export interface State {
  song: Song;
  undoStack: Array<Invertible>;
  redoStack: Array<Invertible>;
}
