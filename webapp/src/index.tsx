import React, { useState } from "react";
import ReactDOM from "react-dom";

import "normalize.css";
import "@blueprintjs/core/lib/css/blueprint.css";
import "./app.css";
import { Card } from "@blueprintjs/core";
import Toolbar, { Tool } from "./toolbar";

// @ts-ignore
import("@blueprintjs/icons/lib/css/blueprint-icons.css");

const HelloRenderer = React.lazy(() => import("./hello_renderer"));

function App() {
  const [tool, setTool] = useState("notes" as Tool);
  const [sixteenth, setSixteenth] = useState(false);
  const [tuplets, setTuplets] = useState(false);

  return (
    <React.Fragment>
      <Toolbar
        tool={tool}
        onSetTool={setTool}
        canUndo={false}
        sixteenth={sixteenth}
        onSixteenthChanged={setSixteenth}
        tuplets={tuplets}
        onTupletChanged={setTuplets}
        onUndo={() => alert("undo")}
        onSave={() => alert("save")}
        onOpen={() => alert("open")}
      />
      <div className="six-note-editor">
        <Card className="six-note-editor-noteview" elevation={1}>
          <React.Suspense
            fallback={<div className="six-note-editor-noteview-placeholder" />}
          >
            <HelloRenderer />
          </React.Suspense>
        </Card>
      </div>
    </React.Fragment>
  );
}

ReactDOM.render(<App />, document.getElementById("app"));
