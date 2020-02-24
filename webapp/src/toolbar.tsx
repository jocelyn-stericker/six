import React from "react";
import cx from "classnames";

import { SAVE, OPEN, UNDO, RESET } from "./toolbar_icons";
import LazyTooltip from "./blueprint/lazy_tooltip";
import "./toolbar.css";

export type Tool = "notes" | "bars" | "select";

export interface Props {
  canUndo: boolean;

  onSave: () => void;
  onOpen: () => void;
  onUndo: () => void;
  onReset: () => void;
}

export default function Toolbar(props: Props) {
  return (
    <div className="six-note-toolbar">
      <div className="six-note-toolbar-spacer" />
      <LazyTooltip
        position="right"
        content={<>Save as PDF&hellip;</>}
        hoverOpenDelay={0}
        transitionDuration={0}
      >
        <div className="six-note-toolbar-action" onClick={props.onSave}>
          <svg viewBox="0 0 20 20" width="100%">
            <path d={SAVE} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip
        position="right"
        content={<>Open Six Eight PDF&hellip;</>}
        hoverOpenDelay={0}
        transitionDuration={0}
      >
        <div className="six-note-toolbar-action" onClick={props.onOpen}>
          <svg viewBox="0 0 20 20" width="100%">
            <path d={OPEN} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip
        position="right"
        content={<>Reset to Default Document</>}
        hoverOpenDelay={0}
        transitionDuration={0}
      >
        <div className="six-note-toolbar-action" onClick={props.onReset}>
          <svg viewBox="0 0 20 20" width="100%">
            <path d={RESET} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip
        position="right"
        content={<>Undo</>}
        hoverOpenDelay={0}
        transitionDuration={0}
      >
        <div
          className={cx(
            "six-note-toolbar-action",
            !props.canUndo && "six-note-toolbar-action-disabled",
          )}
          onClick={props.onUndo}
        >
          <svg viewBox="0 0 20 20" width="100%">
            <path d={UNDO} onClick={props.onUndo} onTouchStart={props.onUndo} />
          </svg>
        </div>
      </LazyTooltip>
      <div className="six-note-toolbar-title">
        Six
        <br />
        Eight
      </div>
    </div>
  );
}
