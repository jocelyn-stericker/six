import React, { useRef } from "react";
import cx from "classnames";

import "normalize.css";

import "./global.scss";
import css from "./app.module.scss";
import {
  getInitialState,
  load,
  redo,
  reduce,
  reset,
  undo,
  update,
} from "./store";
import About from "./cards/about";
import Meta from "./cards/meta";
import { loadPdf, savePdf } from "./file";
import { useLocallyPersistedReducer } from "./local_storage";
import ErrorBoundary from "./error_boundary";

const Hotkeys = React.lazy(() => import("./hotkeys"));
const Editor = React.lazy(() => import("./editor"));
const Navbar = React.lazy(() => import("./navbar"));

export default function App() {
  const [appState, dispatch] = useLocallyPersistedReducer(
    reduce,
    undefined,
    "app",
    getInitialState,
    s => {
      const state = JSON.parse(s);
      state.song = update(state.song);
      return {
        ...getInitialState(),
        song: state.song,
        numChanges: state.numChanges + 1 || 0,
      };
    },
  );
  const sheetEdit = useRef<{ getPDF: () => string }>(null);

  function handleSave() {
    savePdf(
      `${appState.song.global.title || "Untitled"}.sixeight`,
      sheetEdit.current?.getPDF() ?? "",
    );
  }

  function handleOpen() {
    loadPdf().then(result => {
      if ("error" in result) {
        alert(result.error);
      } else {
        dispatch(load(result.song));
      }
    });
  }

  return (
    <ErrorBoundary
      fallback={(err, clearErr) => (
        <div className={cx(css.editorWrapper, css.error)}>
          <h1>Something went wrong.</h1>
          <button onClick={() => location.reload()}>Reload</button>
          <button
            onClick={() => {
              dispatch(reset());
              clearErr();
            }}
          >
            Reset to default document
          </button>
          <h2>Technical details:</h2>
          <pre>{err.toString?.() ?? "No error string"}</pre>
          <pre>{err.stack?.toString?.() ?? "No error stack"}</pre>
        </div>
      )}
    >
      <React.Suspense fallback={<div className={css.navbarLoading} />}>
        <Navbar
          onTrash={() => dispatch(reset())}
          onSave={handleSave}
          onOpen={handleOpen}
        />
      </React.Suspense>
      <About />
      <Meta appState={appState} dispatch={dispatch} />
      <div className={css.editorWrapper}>
        <React.Suspense fallback={null}>
          <Hotkeys
            onUndo={() => dispatch(undo())}
            onRedo={() => dispatch(redo())}
          />
        </React.Suspense>
        <React.Suspense
          fallback={
            <div className={css.editor}>
              <div className={css.editorPlaceholder} />
            </div>
          }
        >
          <Editor ref={sheetEdit} appState={appState} dispatch={dispatch} />
        </React.Suspense>
      </div>
    </ErrorBoundary>
  );
}
