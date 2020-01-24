import React, { useLayoutEffect, useState, useEffect } from "react";
import { Render, render } from "./reconciler";

export { Render, NoteValue, Barline } from "./reconciler";

interface Props {
  children: any;
}

export default function SheetMusicView(props: Props) {
  // create/destroy Rust container
  const [container] = useState(() => Render.new());
  useEffect(() => {
    return () => {
      container.free();
    };
  }, [container]);

  // render loop
  const [svg, setSvg] = useState("");

  useLayoutEffect(() => {
    render(props.children, container);
    container.exec();
    console.log(container.stencils());
    console.log(container.stencil_maps());
    setSvg(container.print_for_demo());
  }, [container, props.children]);

  return <div dangerouslySetInnerHTML={{ __html: svg }} />;
}
