import React, { useRef } from "react";

import "normalize.css";

import "./global.scss";
import css from "./app.module.scss";
import { getInitialState, load, redo, reduce, reset, undo } from "./store";
import About from "./cards/about";
import Meta from "./cards/meta";
import { loadPdf, savePdf } from "./file";
import { useLocallyPersistedReducer } from "./local_storage";

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
    <React.Fragment>
      <React.Suspense fallback={<div className={css.navbarLoading} />}>
        <Navbar
          onTrash={() => dispatch(reset())}
          onSave={handleSave}
          onOpen={handleOpen}
        />
      </React.Suspense>
      <About />
      <Meta appState={appState} dispatch={dispatch} />
      <div className={css.editor}>
        <React.Suspense fallback={null}>
          <Hotkeys
            onUndo={() => dispatch(undo())}
            onRedo={() => dispatch(redo())}
          />
        </React.Suspense>
        <div className={css.noteview}>
          <React.Suspense
            fallback={<div className={css.noteviewPlaceholder} />}
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
