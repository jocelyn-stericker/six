import { Bar, State } from "./state";
import { Action, Invertible } from "./actions";

export function getInitialState(): State {
  return {
    song: {
      v: 1,
      global: {
        title: "",
        author: "",
        between: [
          {
            ts: [4, 4],
            ks: 0,
            clef: "g",
          },
        ],
        pickupSkip: undefined,
      },
      part: {
        bars: Array(9)
          .fill(null)
          .map((_, i) => ({
            notes: [],
            barline: i === 8 ? "final" : "normal",
          })),
      },
    },
    undoStack: [],
    redoStack: [],
  };
}

function apply(state: State, action: Invertible) {
  switch (action.type) {
    case "REMOVE_NOTE": {
      const { barIdx, startTime } = action;
      const barObj = state.song.part.bars[barIdx];
      if (!barObj) {
        return;
      }
      barObj.notes = barObj.notes.filter(
        notes =>
          notes.startTime[0] !== startTime[0] ||
          notes.startTime[1] !== startTime[1],
      );
      break;
    }
    case "ADD_NOTE": {
      const { barIdx, startTime, divisions, pitch } = action;
      const barObj = state.song.part.bars[barIdx];
      if (!barObj) {
        return;
      }
      barObj.notes.push({
        startTime,
        divisions,
        pitch,
      });
      break;
    }
    case "ADD_BAR": {
      state.song.part.bars[state.song.part.bars.length - 1].barline = "normal";
      state.song.part.bars.splice(action.barIdx, 0, action.bar);
      state.song.part.bars[state.song.part.bars.length - 1].barline = "final";
      state.song.global.between.splice(action.barIdx + 1, 0, undefined);
      break;
    }
    case "REMOVE_BAR": {
      state.song.part.bars[state.song.part.bars.length - 1].barline = "normal";
      state.song.part.bars.splice(action.barIdx, 1);
      state.song.part.bars[state.song.part.bars.length - 1].barline = "final";
      const origFirst = state.song.global.between[0];
      state.song.global.between.splice(action.barIdx, 1);
      state.song.global.between[0] = {
        ...state.song.global.between[0],
        ...origFirst,
      };
      break;
    }
    case "SET_BAR_COUNT": {
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
      break;
    }
    case "SET_CLEF": {
      state.song.global.between[action.beforeBar] = {
        ...state.song.global.between[action.beforeBar],
        clef: action.clef,
      };
      break;
    }
    case "SET_TS": {
      const {
        ts,
        barKeepCount,
        barAddCount,
        barRemoveCount,
        after,
        beforeBar,
      } = action;
      let tsInto = null;
      for (let i = 0; i < beforeBar && !tsInto; i += 1) {
        tsInto = state.song.global.between[i]?.ts;
      }

      const origFirst = state.song.global.between[beforeBar] || {};
      state.song.global.between.splice(
        beforeBar,
        barRemoveCount,
        ...Array(barAddCount).fill(undefined),
      );
      state.song.global.between[beforeBar] = {
        ...origFirst,
        ...state.song.global.between[beforeBar],
        ts:
          tsInto && ts[0] === tsInto[0] && ts[1] && tsInto[1] ? undefined : ts,
      };
      let endIdx = beforeBar + barKeepCount + barAddCount - barRemoveCount;
      // TODO: keep clef/ks if also changed.
      // TODO: I'm not sure if this is right.
      if (endIdx >= beforeBar) {
        state.song.global.between[endIdx] = {
          ts:
            after && (after[0] !== ts[0] || after[1] !== ts[1])
              ? [after[0], after[1]]
              : undefined,
        };
      }
      state.song.part.bars.splice(
        beforeBar,
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
      break;
    }
    case "SET_KS": {
      state.song.global.between[action.beforeBar] = {
        ...state.song.global.between[action.beforeBar],
        ks: action.ks,
      };
      break;
    }
    case "SET_TITLE": {
      state.song.global.title = action.title;
      break;
    }
    case "SET_AUTHOR": {
      state.song.global.author = action.author;
      break;
    }
    case "SET_PICKUP": {
      state.song.global.pickupSkip = action.pickupSkip;
      break;
    }
  }
}

function invert(action: Invertible): Invertible {
  switch (action.type) {
    case "REMOVE_NOTE":
      return {
        type: "ADD_NOTE",
        barIdx: action.barIdx,
        startTime: action.startTime,
        divisions: action.divisions,
        pitch: action.pitch,
      };
    case "ADD_NOTE":
      return {
        type: "REMOVE_NOTE",
        barIdx: action.barIdx,
        startTime: action.startTime,
        divisions: action.divisions,
        pitch: action.pitch,
      };
    case "SET_TS":
      return {
        type: "SET_TS",
        beforeBar: action.beforeBar,
        ts: action.prevTs,
        prevTs: action.ts,
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
        beforeBar: action.beforeBar,
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
        beforeBar: action.beforeBar,
      };
    case "SET_PICKUP":
      return {
        type: "SET_PICKUP",
        pickupSkip: action.prevPickupSkip,
        prevPickupSkip: action.pickupSkip,
      };
  }
}

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
    case "SET_PICKUP":
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
    case "LOAD": {
      return {
        ...getInitialState(),
        song: action.song,
      };
    }
  }
}
