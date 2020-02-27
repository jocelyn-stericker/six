export type Clef = "g" | "f" | "percussion";

export interface Global {
  tsNum: number;
  tsDen: number;
  ks: number;
  title: string;
  author: string;
  clef: Clef;
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
  | {
      type: "SET_TS";
      num: number;
      den: number;
      prevNum: number;
      prevDen: number;
    }
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
        tsNum: 4,
        tsDen: 4,
        ks: 0,
        clef: "g",
        title: "",
        author: "",
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
  } else if (action.type === "REMOVE_BAR") {
    state.song.part.bars[state.song.part.bars.length - 1].barline = "normal";
    state.song.part.bars.splice(action.barIdx, 1);
    state.song.part.bars[state.song.part.bars.length - 1].barline = "final";
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
    state.song.global.clef = action.clef;
  } else if (action.type === "SET_TS") {
    const { num, den } = action;
    state.song.global.tsNum = num;
    state.song.global.tsDen = den;
  } else if (action.type === "SET_KS") {
    state.song.global.ks = action.ks;
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
