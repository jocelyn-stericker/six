import React, { useReducer, useRef } from "react";

import "normalize.css";
import "./app.css";
import { getInitialState, load, redo, reduce, reset, undo } from "./store";
import About from "./about";
import Meta from "./meta";
import loadPdf from "./load_pdf";

const AppHotkeys = React.lazy(() => import("./app_hotkeys"));
const SheetEdit = React.lazy(() => import("./sheet_edit"));
const Navbar = React.lazy(() => import("./navbar"));

export default function App() {
  const [appState, dispatch] = useReducer(reduce, undefined, getInitialState);
  const sheetEdit = useRef<{ saveAsPDF: () => void }>(null);

  return (
    <React.Fragment>
      <React.Suspense
        fallback={
          <div style={{ width: "100%", height: 50, background: "white" }} />
        }
      >
        <Navbar
          onTrash={() => dispatch(reset())}
          onSave={() => sheetEdit.current?.saveAsPDF()}
          onOpen={() => {
            loadPdf().then(result => {
              if ("error" in result) {
                alert(result.error);
              } else {
                dispatch(load(result.song));
              }
            });
          }}
        />
      </React.Suspense>
      <About />
      <Meta appState={appState} dispatch={dispatch} />
      <div className="six-note-editor">
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
            <SheetEdit
              ref={sheetEdit}
              appState={appState}
              dispatch={dispatch}
            />
          </React.Suspense>
        </div>
      </div>
    </React.Fragment>
  );
}
