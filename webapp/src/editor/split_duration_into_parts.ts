import { State, TiedNote } from "../store";
import { NativeSixDom } from "../scene";

export interface NoteAddPatch {
  barIdx: number;
  startTime: [number, number];
  divisions: TiedNote;
}

export function count(noteValue: number, dots: number) {
  let base = Math.pow(2, noteValue);
  let total = base;
  for (let i = 0; i < dots; ++i) {
    total += base / 2;
    base / 2;
  }

  return total;
}

export default function splitDurationIntoParts(
  render: NativeSixDom | null,
  appState: State,
  barEntity: number | null,
  barIdx: number,
  time: [number, number] | null,
  insertionDuration: [number, number],
): NoteAddPatch | null {
  if (!render) {
    return null;
  }
  if (!time) {
    return null;
  }
  if (!barEntity) {
    return null;
  }

  const rawDivisions = render.split_note(
    barEntity,
    time[0],
    time[1],
    insertionDuration[0],
    insertionDuration[1],
  );
  const start = time[0] / time[1];

  const divisions: TiedNote = [];
  let end = start;
  for (let i = 0; i < rawDivisions.length; i += 4) {
    end += count(rawDivisions[i], rawDivisions[i + 1]);
    divisions.push({
      noteValue: rawDivisions[i],
      dots: rawDivisions[i + 1],
      startTime: [rawDivisions[i + 2], rawDivisions[i + 3]],
    });
  }
  if (!divisions.length) {
    return null;
  }

  if (
    appState.song.part.bars[barIdx].notes.some(note => {
      let noteStart = note.startTime[0] / note.startTime[1];
      let noteEnd =
        noteStart +
        note.divisions.reduce(
          (sum, { noteValue, dots }) => sum + count(noteValue, dots),
          0,
        );
      // TODO: check if this note is in the middle of the proposed one.
      return (
        (start <= noteStart && end > noteStart) ||
        (start < noteEnd && end >= noteEnd)
      );
    })
  ) {
    return null;
  }

  return {
    barIdx,
    startTime: [time[0], time[1]],
    divisions,
  };
}
