import React, { useRef } from "react";

import "normalize.css";
import "./app.css";
import { getInitialState, load, redo, reduce, reset, undo } from "./store";
import About from "./cards/about";
import Meta from "./cards/meta";
import { loadPdf, savePdf } from "./file";
import { useLocallyPersistedReducer } from "./localStorage";

const Hotkeys = React.lazy(() => import("./hotkeys"));
const SheetEdit = React.lazy(() => import("./editor"));
const Navbar = React.lazy(() => import("./navbar"));

export default function App() {
  const [appState, dispatch] = useLocallyPersistedReducer(
    reduce,
    undefined,
    "app",
    getInitialState,
  );
  const sheetEdit = useRef<{ getPDF: () => string }>(null);

  return (
    <React.Fragment>
      <React.Suspense
        fallback={
          <div style={{ width: "100%", height: 50, background: "white" }} />
        }
      >
        <Navbar
          onTrash={() => dispatch(reset())}
          onSave={() => {
            savePdf(
              `${appState.song.global.title || "Untitled"}.sixeight`,
              sheetEdit.current?.getPDF() ?? "",
            );
          }}
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
          <Hotkeys
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
