export interface Global {
  tsNum: number;
  tsDen: number;
}

export interface Note {
  startNum: number;
  startDen: number;
  duration: number;
  dots: number;
}

export interface Bar {
  barline: "normal" | "final";
  notes: Array<Note>;
}

export interface Part {
  bars: Array<Bar>;
}

export interface Song {
  v: "0.10.0";
  global: Global;
  part: Part;
}

type ApplyInvertAction =
  | {
      type: "REMOVE_NOTE";
      bar: number;
      num: number;
      den: number;
      duration: number;
      dots: number;
    }
  | {
      type: "ADD_NOTE";
      bar: number;
      num: number;
      den: number;
      duration: number;
      dots: number;
    }
  | {
      type: "SET_TS";
      num: number;
      den: number;
      prevNum: number;
      prevDen: number;
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
      v: "0.10.0",
      global: {
        tsNum: 4,
        tsDen: 4
      },
      part: {
        bars: [
          {
            notes: [],
            barline: "normal"
          },
          {
            notes: [],
            barline: "normal"
          },
          {
            notes: [],
            barline: "normal"
          },
          {
            notes: [],
            barline: "final"
          }
        ]
      }
    },
    undoStack: [],
    redoStack: []
  };
}

function invert(action: ApplyInvertAction): ApplyInvertAction {
  switch (action.type) {
    case "REMOVE_NOTE":
      return {
        type: "ADD_NOTE",
        bar: action.bar,
        num: action.num,
        den: action.den,
        duration: action.duration,
        dots: action.dots
      };
    case "ADD_NOTE":
      return {
        type: "REMOVE_NOTE",
        bar: action.bar,
        num: action.num,
        den: action.den,
        duration: action.duration,
        dots: action.dots
      };
    case "SET_TS":
      return {
        type: "SET_TS",
        num: action.prevNum,
        den: action.prevDen,
        prevNum: action.num,
        prevDen: action.den
      };
  }
}

function apply(state: State, action: ApplyInvertAction) {
  if (action.type === "REMOVE_NOTE") {
    const { bar, num, den, duration, dots } = action;
    const barObj = state.song.part.bars[bar];
    if (!barObj) {
      return;
    }
    barObj.notes = barObj.notes.filter(
      notes =>
        notes.startNum !== num ||
        notes.startDen !== den ||
        notes.duration !== duration ||
        notes.dots !== dots
    );
  } else if (action.type === "ADD_NOTE") {
    const { bar, num, den, duration, dots } = action;
    const barObj = state.song.part.bars[bar];
    if (!barObj) {
      return;
    }
    barObj.notes.push({
      startNum: num,
      startDen: den,
      duration,
      dots
    });
  } else if (action.type === "SET_TS") {
    const { num, den } = action;
    state.song.global.tsNum = num;
    state.song.global.tsDen = den;
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
      apply(state, action);
      state.undoStack.push(action);
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
