import React, { useState, useReducer } from "react";

import "normalize.css";
import "./app.css";
import Toolbar, { Tool } from "./toolbar";
import { reduce, getInitialState } from "./store";
import About from "./about";

const AppHotkeys = React.lazy(() => import("./app_hotkeys"));
const SheetEdit = React.lazy(() => import("./sheet_edit"));

export default function App() {
  const [tool, setTool] = useState("notes" as Tool);
  const [appState, dispatch] = useReducer(reduce, undefined, getInitialState);

  return (
    <React.Fragment>
      <About />
      <div className="six-note-editor">
        <Toolbar
          tool={tool}
          onSetTool={setTool}
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
            onSetTool={setTool}
          />
        </React.Suspense>
        <div className="six-note-editor-noteview">
          <React.Suspense
            fallback={<div className="six-note-editor-noteview-placeholder" />}
          >
            <SheetEdit tool={tool} appState={appState} dispatch={dispatch} />
          </React.Suspense>
        </div>
      </div>
    </React.Fragment>
  );
}
