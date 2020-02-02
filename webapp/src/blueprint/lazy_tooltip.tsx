import React from "react";

const Tooltip = React.lazy(() => import("./tooltip"));

export default function LazyTooltip(
  props: React.ComponentProps<typeof import("@blueprintjs/core").Tooltip> & {
    children?: any;
  }
) {
  return (
    // @ts-ignore
    <React.Suspense fallback={props.children}>
      <Tooltip {...props} />
    </React.Suspense>
  );
}
