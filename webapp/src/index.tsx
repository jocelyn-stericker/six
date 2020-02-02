import React, { useState, useReducer } from "react";
import ReactDOM from "react-dom";

import "normalize.css";
import "./app.css";
import Toolbar, { Tool } from "./toolbar";
import { reduce, getInitialState } from "./store";

const SixHotkeys = React.lazy(() => import("./hotkeys"));
const HelloRenderer = React.lazy(() => import("./hello_renderer"));

function App() {
  const [tool, setTool] = useState("notes" as Tool);
  const [appState, dispatch] = useReducer(reduce, undefined, getInitialState);

  return (
    <React.Fragment>
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
          <SixHotkeys
            onUndo={() => dispatch({ type: "UNDO" })}
            onRedo={() => dispatch({ type: "REDO" })}
          />
        </React.Suspense>
        <div className="six-note-editor-noteview">
          <React.Suspense
            fallback={<div className="six-note-editor-noteview-placeholder" />}
          >
            <HelloRenderer appState={appState} dispatch={dispatch} />
          </React.Suspense>
        </div>
      </div>
    </React.Fragment>
  );
}

ReactDOM.render(<App />, document.getElementById("app"));
