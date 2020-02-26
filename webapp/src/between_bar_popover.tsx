import React from "react";
import { Popover, Menu, MenuItem, MenuDivider } from "@blueprintjs/core";
import { Clef } from "./store";

export interface Props {
  children: any;
  tsNum: number;
  tsDen: number;
  setClef: (clef: Clef) => void;
  setKs: (ks: number) => void;
  setTs: (ts: [number, number]) => void;
  onInsertBarRight: () => void;
  onRemoveBarRight: () => void | null;
}

export default function BetweenBarPopover({
  children,
  setClef,
  setKs,
  setTs,
  tsNum,
  tsDen,
  onInsertBarRight,
  onRemoveBarRight,
}: Props) {
  return (
    <Popover
      position="right-top"
      popoverClassName="bp3-dark"
      captureDismiss={true}
      minimal={true}
      modifiers={
        {
          preventOverflow: true,
        } as any
      }
      interactionKind="click"
      hasBackdrop={true}
      content={
        <Menu large={true}>
          <MenuItem
            icon="blank"
            text="Insert Bar Right"
            onClick={onInsertBarRight}
          />
          <MenuItem
            icon="blank"
            text="Remove Next Bar"
            disabled={!onRemoveBarRight}
            onClick={onRemoveBarRight}
          />
          <MenuDivider />
          <MenuItem
            text="Edit Time Signature"
            icon={"ts" as any}
            popoverProps={{
              interactionKind: "hover",
              hoverCloseDelay: 350,
              hoverOpenDelay: 350,
            }}
          >
            <li className="bp3-menu-header">
              <h6 className="bp3-heading">Simple</h6>
            </li>
            <MenuItem
              icon="blank"
              text="4/4"
              active={tsNum === 4 && tsDen === 4}
              onClick={() => setTs([4, 4])}
            />
            <MenuItem
              icon="blank"
              text="2/2"
              active={tsNum === 2 && tsDen === 2}
              onClick={() => setTs([2, 2])}
            />
            <MenuItem
              icon="blank"
              text="2/4"
              active={tsNum === 2 && tsDen === 4}
              onClick={() => setTs([2, 4])}
            />
            <MenuItem
              icon="blank"
              text="4/8"
              active={tsNum === 4 && tsDen === 8}
              onClick={() => setTs([4, 8])}
            />
            <MenuDivider />
            <MenuItem
              icon="blank"
              text="3/4"
              active={tsNum === 3 && tsDen === 4}
              onClick={() => setTs([3, 4])}
            />
            <MenuItem
              icon="blank"
              text="3/8"
              active={tsNum === 3 && tsDen === 8}
              onClick={() => setTs([3, 8])}
            />
            <li className="bp3-menu-header">
              <h6 className="bp3-heading">Compound</h6>
            </li>
            <MenuItem
              icon="blank"
              text="6/8"
              active={tsNum === 6 && tsDen === 8}
              onClick={() => setTs([6, 8])}
            />
            <MenuItem
              icon="blank"
              text="6/4"
              active={tsNum === 6 && tsDen === 4}
              onClick={() => setTs([6, 4])}
            />
            <MenuItem
              icon="blank"
              text="6/16"
              active={tsNum === 6 && tsDen === 16}
              onClick={() => setTs([6, 16])}
            />
            <MenuDivider />
            <MenuItem
              icon="blank"
              text="9/8"
              active={tsNum === 9 && tsDen === 8}
              onClick={() => setTs([9, 8])}
            />
            <MenuItem
              icon="blank"
              text="12/8"
              active={tsNum === 12 && tsDen === 8}
              onClick={() => setTs([12, 8])}
            />
            <MenuDivider />
            <MenuItem text="No change" icon="delete" />
          </MenuItem>
          <MenuItem
            text="Edit Clef"
            icon={"bass" as any}
            popoverProps={{
              interactionKind: "hover",
              hoverCloseDelay: 350,
              hoverOpenDelay: 350,
            }}
          >
            <MenuItem
              onClick={() => setClef("g")}
              text="Treble"
              icon={"treble" as any}
            />
            <MenuItem
              onClick={() => setClef("f")}
              text="Bass"
              icon={"bass" as any}
            />
            <MenuItem
              onClick={() => setClef("percussion")}
              text="Percussion"
              icon={"percussion" as any}
            />
            <MenuDivider />
            <MenuItem text="No change" icon="delete" />
          </MenuItem>
          <MenuItem
            text="Edit Key Signature"
            icon={"ks" as any}
            popoverProps={{
              interactionKind: "hover",
              hoverCloseDelay: 350,
              hoverOpenDelay: 350,
            }}
          >
            <MenuItem
              icon="blank"
              onClick={() => setKs(-6)}
              text="G♭ Major / e♭ minor (6♭)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(-5)}
              text="D♭ Major / b♭ minor (5♭)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(-4)}
              text="A♭ Major / f minor (4♭)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(-3)}
              text="E♭ Major / c minor (3♭)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(-2)}
              text="B♭ Major / g minor (2♭)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(-1)}
              text="F Major / d minor (♭)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(0)}
              text="C Major / a minor"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(1)}
              text="G Major / e minor (♯)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(2)}
              text="D Major / b minor (2♯)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(3)}
              text="A Major / f♯ minor (3♯)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(4)}
              text="E Major / c♯ minor (4♯)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(5)}
              text="B Major / g♯ minor (5♯)"
            />
            <MenuItem
              icon="blank"
              onClick={() => setKs(6)}
              text="F♯ Major / d♯ minor (6♯)"
            />
            <MenuDivider />
            <MenuItem text="No change" icon="delete" />
          </MenuItem>
        </Menu>
      }
    >
      {children}
    </Popover>
  );
}
