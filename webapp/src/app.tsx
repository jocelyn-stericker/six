import React, { useReducer } from "react";

import "normalize.css";
import "./app.css";
import { getInitialState, redo, reduce, undo } from "./store";
import About from "./about";
import Meta from "./meta";

const AppHotkeys = React.lazy(() => import("./app_hotkeys"));
const SheetEdit = React.lazy(() => import("./sheet_edit"));

export default function App() {
  const [appState, dispatch] = useReducer(reduce, undefined, getInitialState);

  return (
    <React.Fragment>
      <About
        title={appState.song.global.title}
        author={appState.song.global.author}
      />
      <Meta appState={appState} dispatch={dispatch} />
      <div className="six-note-editor">
        <h2>Notes</h2>
        <React.Suspense fallback={null}>
          <AppHotkeys
            onUndo={() => dispatch(undo())}
            onRedo={() => dispatch(redo())}
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
