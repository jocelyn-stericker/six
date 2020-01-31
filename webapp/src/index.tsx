import React, { useState } from "react";
import ReactDOM from "react-dom";

import "normalize.css";
import "./app.css";
import Toolbar, { Tool } from "./toolbar";

const HelloRenderer = React.lazy(() => import("./hello_renderer"));

function App() {
  const [tool, setTool] = useState("notes" as Tool);

  return (
    <React.Fragment>
      <div className="six-note-editor">
        <Toolbar
          tool={tool}
          onSetTool={setTool}
          canUndo={false}
          onUndo={() => alert("undo")}
          onSave={() => alert("save")}
          onOpen={() => alert("open")}
          onReset={() => alert("reset")}
        />
        <div className="six-note-editor-noteview">
          <React.Suspense
            fallback={<div className="six-note-editor-noteview-placeholder" />}
          >
            <HelloRenderer />
          </React.Suspense>
        </div>
      </div>
    </React.Fragment>
  );
}

ReactDOM.render(<App />, document.getElementById("app"));
