import React from "react";
import cx from "classnames";

import { NOTES, SELECT, EDIT, SAVE, OPEN, UNDO, RESET } from "./toolbar_icons";
import LazyTooltip from "./lazy_tooltip";

export type Tool = "notes" | "bars" | "select";

export interface Props {
  tool: Tool;
  canUndo: boolean;

  onSetTool: (tool: Tool) => void;
  onSave: () => void;
  onOpen: () => void;
  onUndo: () => void;
  onReset: () => void;
}

export default function Toolbar(props: Props) {
  return (
    <div className="six-note-toolbar">
      <LazyTooltip position="right" content="Add Notes">
        <div
          className={cx(
            "six-note-toolbar-mode",
            props.tool === "notes" && "six-note-toolbar-mode-selected"
          )}
          onClick={() => props.onSetTool("notes")}
        >
          <svg viewBox="0 0 20 20" width="100%">
            <path d={NOTES} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip
        position="right"
        content={<>Select, Decorate, and Delete Notes</>}
      >
        <div
          className={cx(
            "six-note-toolbar-mode",
            props.tool === "select" && "six-note-toolbar-mode-selected"
          )}
          onClick={() => props.onSetTool("select")}
        >
          <svg viewBox="0 0 20 20" width="100%">
            <path d={SELECT} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip position="right" content={<>Edit Bars and Signatures</>}>
        <div
          className={cx(
            "six-note-toolbar-mode",
            props.tool === "bars" && "six-note-toolbar-mode-selected"
          )}
          onClick={() => props.onSetTool("bars")}
        >
          <svg viewBox="0 0 20 20" width="100%">
            <path d={EDIT} />
          </svg>
        </div>
      </LazyTooltip>
      <div className="six-note-toolbar-spacer" />
      <LazyTooltip position="right" content={<>Save as PDF&hellip;</>}>
        <div className="six-note-toolbar-action" onClick={props.onSave}>
          <svg viewBox="0 0 20 20" width="100%">
            <path d={SAVE} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip position="right" content={<>Open Six Eight PDF&hellip;</>}>
        <div className="six-note-toolbar-action" onClick={props.onOpen}>
          <svg viewBox="0 0 20 20" width="100%">
            <path d={OPEN} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip position="right" content={<>Reset to Default Document</>}>
        <div className="six-note-toolbar-action">
          <svg viewBox="0 0 20 20" width="100%">
            <path d={RESET} onClick={props.onReset} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip position="right" content={<>Undo</>}>
        <div
          className={cx(
            "six-note-toolbar-action",
            !props.canUndo && "six-note-toolbar-action-disabled"
          )}
          onClick={props.onUndo}
        >
          <svg viewBox="0 0 20 20" width="100%">
            <path d={UNDO} onClick={props.onUndo} />
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
