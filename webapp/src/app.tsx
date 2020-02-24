import React, { useReducer } from "react";

import "normalize.css";
import "./app.css";
import Toolbar from "./toolbar";
import { reduce, getInitialState } from "./store";
import About from "./about";
import Meta from "./meta";

const AppHotkeys = React.lazy(() => import("./app_hotkeys"));
const SheetEdit = React.lazy(() => import("./sheet_edit"));

export default function App() {
  const [appState, dispatch] = useReducer(reduce, undefined, getInitialState);

  return (
    <React.Fragment>
      <About />
      <Meta appState={appState} dispatch={dispatch} />
      <div className="six-note-editor">
        <Toolbar
          canUndo={appState.undoStack.length > 0}
          onUndo={() => dispatch({ type: "UNDO" })}
          onSave={() => alert("save")}
          onOpen={() => alert("open")}
          onReset={() => dispatch({ type: "RESET" })}
        />
        <React.Suspense fallback={null}>
          <AppHotkeys
            onUndo={() => dispatch({ type: "UNDO" })}
            onRedo={() => dispatch({ type: "REDO" })}
          />
        </React.Suspense>
        <div className="six-note-editor-noteview">
          <React.Suspense
            fallback={<div className="six-note-editor-noteview-placeholder" />}
          >
            <SheetEdit appState={appState} dispatch={dispatch} />
          </React.Suspense>
        </div>
      </div>
    </React.Fragment>
  );
}
