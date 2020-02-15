import React from "react";
import cx from "classnames";

import { NOTES, EDIT, SAVE, OPEN, UNDO, RESET } from "./toolbar_icons";
import LazyTooltip from "./blueprint/lazy_tooltip";
import "./toolbar.css";

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
      <LazyTooltip
        position="right"
        content={
          <>
            Mode: Add Notes (
            <strong>
              <u>n</u>
            </strong>
            )
          </>
        }
        hoverOpenDelay={0}
        transitionDuration={0}
        lazy={false}
      >
        <div
          className={cx(
            "six-note-toolbar-mode",
            props.tool === "notes" && "six-note-toolbar-mode-selected",
          )}
          onClick={() => props.onSetTool("notes")}
          onTouchStart={() => props.onSetTool("notes")}
        >
          <svg viewBox="0 0 20 20" width="100%">
            <path d={NOTES} />
          </svg>
        </div>
      </LazyTooltip>
      <LazyTooltip
        position="right"
        content={
          <>
            Mode: Edit Bars and Signatures (
            <strong>
              <u>b</u>
            </strong>
            )
          </>
        }
        hoverOpenDelay={0}
        transitionDuration={0}
      >
        <div
          className={cx(
            "six-note-toolbar-mode",
            props.tool === "bars" && "six-note-toolbar-mode-selected",
          )}
          onClick={() => props.onSetTool("bars")}
          onTouchStart={() => props.onSetTool("bars")}
        >
          <svg viewBox="0 0 20 20" width="100%">
            <path d={EDIT} />
          </svg>
        </div>
      </LazyTooltip>
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
