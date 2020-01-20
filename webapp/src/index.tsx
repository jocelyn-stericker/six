import React from "react";
import ReactDOM from "react-dom";

const HelloRenderer = React.lazy(() => import("./hello_renderer"));

ReactDOM.render(
  <span>
    <React.Suspense fallback={<div>...</div>}>
      <HelloRenderer />
    </React.Suspense>
  </span>,
  document.getElementById("app")
);
