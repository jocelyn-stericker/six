export type Clef = "g" | "f" | "percussion";

export interface Between {
  ts: [number, number];
  ks: number;
  clef: Clef;
}

export interface Global {
  title: string;
  author: string;
  between: { [0]: Between; [key: number]: Partial<Between> };
}

export type TiedNote = Array<{
  noteValue: number;
  dots: number;
  startNum: number;
  startDen: number;
}>;

export interface Note {
  startNum: number;
  startDen: number;
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

export interface ActionSetTs {
  type: "SET_TS";
  num: number;
  den: number;
  prevNum: number;
  prevDen: number;

  // We never change the ts of existing music.
  // Instead, we add bars when one is already filled.
  barAddCount: number;
  barKeepCount: number;
  // For undo.
  barRemoveCount: number;

  after: [number, number] | null;
}

type ApplyInvertAction =
  | {
      type: "REMOVE_NOTE";
      barIdx: number;
      startNum: number;
      startDen: number;
      divisions: TiedNote;
    }
  | {
      type: "ADD_NOTE";
      barIdx: number;
      startNum: number;
      startDen: number;
      divisions: TiedNote;
    }
  | ActionSetTs
  | {
      type: "SET_KS";
      ks: number;
      prevKs: number;
    }
  | {
      type: "SET_CLEF";
      clef: Clef;
      prevClef: Clef;
    }
  | {
      type: "ADD_BAR";
      barIdx: number;
      bar: Bar;
    }
  | {
      type: "REMOVE_BAR";
      barIdx: number;
      bar: Bar;
    }
  | {
      type: "SET_BAR_COUNT";
      count: number;
      prevCount: number;
    }
  | {
      type: "SET_TITLE";
      title: string;
      prevTitle: string;
    }
  | {
      type: "SET_AUTHOR";
      author: string;
      prevAuthor: string;
    };

export type Action =
  | ApplyInvertAction
  | {
      type: "UNDO";
    }
  | {
      type: "REDO";
    }
  | {
      type: "RESET";
    };

export interface State {
  song: Song;
  undoStack: Array<ApplyInvertAction>;
  redoStack: Array<ApplyInvertAction>;
}

export function getInitialState(): State {
  return {
    song: {
      v: "0.1.0",
      global: {
        title: "",
        author: "",
        between: {
          0: {
            ts: [4, 4],
            ks: 0,
            clef: "g",
          },
        },
      },
      part: {
        bars: [
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "normal",
          },
          {
            notes: [],
            barline: "final",
          },
        ],
      },
    },
    undoStack: [],
    redoStack: [],
  };
}

function invert(action: ApplyInvertAction): ApplyInvertAction {
  switch (action.type) {
    case "REMOVE_NOTE":
      return {
        type: "ADD_NOTE",
        barIdx: action.barIdx,
        startNum: action.startNum,
        startDen: action.startDen,
        divisions: action.divisions,
      };
    case "ADD_NOTE":
      return {
        type: "REMOVE_NOTE",
        barIdx: action.barIdx,
        startNum: action.startNum,
        startDen: action.startDen,
        divisions: action.divisions,
      };
    case "SET_TS":
      return {
        type: "SET_TS",
        num: action.prevNum,
        den: action.prevDen,
        prevNum: action.num,
        prevDen: action.den,
        barAddCount: action.barRemoveCount,
        barKeepCount: action.barKeepCount,
        barRemoveCount: action.barAddCount,
        after: action.after,
      };
    case "SET_KS":
      return {
        type: "SET_KS",
        ks: action.prevKs,
        prevKs: action.ks,
      };
    case "ADD_BAR":
      return {
        type: "REMOVE_BAR",
        barIdx: action.barIdx,
        bar: action.bar,
      };
    case "REMOVE_BAR":
      return {
        type: "ADD_BAR",
        barIdx: action.barIdx,
        bar: action.bar,
      };
    case "SET_BAR_COUNT":
      return {
        type: "SET_BAR_COUNT",
        count: action.prevCount,
        prevCount: action.count,
      };
    case "SET_AUTHOR":
      return {
        type: "SET_AUTHOR",
        author: action.prevAuthor,
        prevAuthor: action.author,
      };
    case "SET_TITLE":
      return {
        type: "SET_TITLE",
        title: action.prevTitle,
        prevTitle: action.title,
      };
    case "SET_CLEF":
      return {
        type: "SET_CLEF",
        clef: action.prevClef,
        prevClef: action.clef,
      };
  }
}

function apply(state: State, action: ApplyInvertAction) {
  if (action.type === "REMOVE_NOTE") {
    const { barIdx, startNum, startDen } = action;
    const barObj = state.song.part.bars[barIdx];
    if (!barObj) {
      return;
    }
    barObj.notes = barObj.notes.filter(
      notes => notes.startNum !== startNum || notes.startDen !== startDen,
    );
  } else if (action.type === "ADD_NOTE") {
    const { barIdx, startNum, startDen, divisions } = action;
    const barObj = state.song.part.bars[barIdx];
    if (!barObj) {
      return;
    }
    barObj.notes.push({
      startNum,
      startDen,
      divisions,
    });
  } else if (action.type === "ADD_BAR") {
    state.song.part.bars[state.song.part.bars.length - 1].barline = "normal";
    state.song.part.bars.splice(action.barIdx, 0, action.bar);
    state.song.part.bars[state.song.part.bars.length - 1].barline = "final";
    let newBetween: { [0]: Between; [key: number]: Partial<Between> } = {
      [0]: state.song.global.between[0],
    };
    Object.entries(state.song.global.between).map(([key, val]) => {
      let num = parseInt(key);
      if (num > action.barIdx) {
        newBetween[num + 1] = val;
      } else {
        newBetween[num] = val;
      }
    });
    state.song.global.between = newBetween;
  } else if (action.type === "REMOVE_BAR") {
    state.song.part.bars[state.song.part.bars.length - 1].barline = "normal";
    state.song.part.bars.splice(action.barIdx, 1);
    state.song.part.bars[state.song.part.bars.length - 1].barline = "final";
    let newBetween: { [0]: Between; [key: number]: Partial<Between> } = {
      [0]: state.song.global.between[0],
    };
    Object.entries(state.song.global.between).map(([key, val]) => {
      let num = parseInt(key);
      if (num > action.barIdx) {
        newBetween[num - 1] = val;
      } else {
        newBetween[num] = val;
      }
    });
    state.song.global.between = newBetween;
  } else if (action.type === "SET_BAR_COUNT") {
    state.song.part.bars[state.song.part.bars.length - 1].barline = "normal";
    while (state.song.part.bars.length < action.count) {
      state.song.part.bars.push({
        notes: [],
        barline: "normal",
      });
    }
    while (state.song.part.bars.length > action.count) {
      state.song.part.bars.pop();
    }
    state.song.part.bars[state.song.part.bars.length - 1].barline = "final";
  } else if (action.type === "SET_CLEF") {
    state.song.global.between[0].clef = action.clef;
  } else if (action.type === "SET_TS") {
    const {
      num,
      den,
      barKeepCount,
      barAddCount,
      barRemoveCount,
      after,
    } = action;
    let newBetween: { [0]: Between; [key: number]: Partial<Between> } = {
      [0]: state.song.global.between[0],
    };
    Object.entries(state.song.global.between).map(([key, val]) => {
      let num = parseInt(key);
      num += barAddCount - barRemoveCount;
      if (num >= 0) {
        newBetween[num] = val;
      }
    });
    newBetween[0] = {
      ...state.song.global.between[0],
      ...newBetween[0],
      ts: [num, den],
    };
    let endIdx = barKeepCount + barAddCount - barRemoveCount;
    newBetween[endIdx] = newBetween[endIdx] ?? {};
    newBetween[endIdx].ts =
      after && (after[0] !== num || after[1] !== den)
        ? [after[0], after[1]]
        : undefined;
    state.song.part.bars.splice(
      0,
      barRemoveCount,
      ...Array(barAddCount)
        .fill(null)
        .map(
          () =>
            ({
              notes: [],
              barline: "normal",
            } as Bar),
        ),
    );

    state.song.global.between = newBetween;
  } else if (action.type === "SET_KS") {
    state.song.global.between[0].ks = action.ks;
  } else if (action.type === "SET_TITLE") {
    state.song.global.title = action.title;
  } else if (action.type === "SET_AUTHOR") {
    state.song.global.author = action.author;
  }
}

/**
 * NOTE: state is internally mutable.
 */
export function reduce(state: State, action: Action): State {
  console.debug("action:", JSON.stringify(action));
  switch (action.type) {
    case "REMOVE_NOTE":
    case "ADD_NOTE":
    case "SET_TS":
    case "SET_KS":
    case "SET_CLEF":
    case "ADD_BAR":
    case "REMOVE_BAR":
    case "SET_BAR_COUNT":
      apply(state, action);
      state.undoStack.push(action);
      state.redoStack = [];
      return { ...state };
    case "SET_TITLE":
      apply(state, action);
      {
        const prevAction = state.undoStack[state.undoStack.length - 1];
        if (prevAction && prevAction.type === "SET_TITLE") {
          prevAction.title = action.title;
        } else {
          state.undoStack.push(action);
        }
      }
      state.redoStack = [];
      return { ...state };
    case "SET_AUTHOR":
      apply(state, action);
      {
        const prevAction = state.undoStack[state.undoStack.length - 1];
        if (prevAction && prevAction.type === "SET_AUTHOR") {
          prevAction.author = action.author;
        } else {
          state.undoStack.push(action);
        }
      }
      state.redoStack = [];
      return { ...state };
    case "UNDO":
      {
        const toUndo = state.undoStack.pop();
        if (toUndo != null) {
          apply(state, invert(toUndo));
          state.redoStack.push(toUndo);
        }
      }
      return { ...state };
    case "REDO":
      {
        const toRedo = state.redoStack.pop();
        if (toRedo != null) {
          apply(state, toRedo);
          state.undoStack.push(toRedo);
        }
      }
      return { ...state };
    case "RESET": {
      return getInitialState();
    }
  }
}

export function setTs(
  appState: State,
  [num, den]: [number, number],
): ActionSetTs {
  let barsWithoutContent = 0;
  let after = appState.song.global.between[0].ts;
  while (
    appState.song.part.bars[barsWithoutContent]?.notes.length === 0 &&
    (!barsWithoutContent ||
      !appState.song.global.between[barsWithoutContent]?.ts)
  ) {
    barsWithoutContent += 1;
    after = appState.song.global.between[barsWithoutContent]?.ts ?? after;
  }

  return {
    type: "SET_TS",
    num,
    den,
    prevNum: appState.song.global.between[0].ts[0],
    prevDen: appState.song.global.between[0].ts[1],
    barAddCount: barsWithoutContent === 0 ? 1 : 0,
    barKeepCount: barsWithoutContent,
    barRemoveCount: 0,
    after,
  };
}
