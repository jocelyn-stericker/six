import React from "react";
import { Popover, Menu, MenuItem, MenuDivider } from "@blueprintjs/core";

export interface Props {
  children: any;
  tsNum: number;
  tsDen: number;
  setTs: (ts: [number, number]) => void;
}

export default function BetweenBarPopover({
  children,
  setTs,
  tsNum,
  tsDen
}: Props) {
  return (
    <Popover
      position="bottom"
      content={
        <Menu>
          <MenuItem text="Edit Time Signature">
            <li className="bp3-menu-header">
              <h6 className="bp3-heading">Simple</h6>
            </li>
            <MenuItem
              text="4/4"
              active={tsNum === 4 && tsDen === 4}
              onClick={() => setTs([4, 4])}
            />
            <MenuItem
              text="2/2"
              active={tsNum === 2 && tsDen === 2}
              onClick={() => setTs([2, 2])}
            />
            <MenuItem
              text="2/4"
              active={tsNum === 2 && tsDen === 4}
              onClick={() => setTs([2, 4])}
            />
            <MenuItem
              text="4/8"
              active={tsNum === 4 && tsDen === 8}
              onClick={() => setTs([4, 8])}
            />
            <MenuDivider />
            <MenuItem
              text="3/4"
              active={tsNum === 3 && tsDen === 4}
              onClick={() => setTs([3, 4])}
            />
            <MenuItem
              text="3/8"
              active={tsNum === 3 && tsDen === 8}
              onClick={() => setTs([3, 8])}
            />
            <li className="bp3-menu-header">
              <h6 className="bp3-heading">Compound</h6>
            </li>
            <MenuItem
              text="6/8"
              active={tsNum === 6 && tsDen === 8}
              onClick={() => setTs([6, 8])}
            />
            <MenuItem
              text="6/4"
              active={tsNum === 6 && tsDen === 4}
              onClick={() => setTs([6, 4])}
            />
            <MenuItem
              text="6/16"
              active={tsNum === 6 && tsDen === 16}
              onClick={() => setTs([6, 16])}
            />
            <MenuDivider />
            <MenuItem
              text="9/8"
              active={tsNum === 9 && tsDen === 8}
              onClick={() => setTs([9, 8])}
            />
            <MenuItem
              text="12/8"
              active={tsNum === 12 && tsDen === 8}
              onClick={() => setTs([12, 8])}
            />
            <MenuDivider />
            <MenuItem text="No change" />
          </MenuItem>
          <MenuItem text="Edit Clef">
            <MenuItem text="Treble" />
            <MenuItem text="Bass" />
            <MenuItem text="Percussion" />
            <MenuDivider />
            <MenuItem text="No change" />
          </MenuItem>
          <MenuItem text="Edit Key Signature">
            <MenuItem text="G♭ Major / e♭ minor (6♭)" />
            <MenuItem text="D♭ Major / b♭ minor (5♭)" />
            <MenuItem text="A♭ Major / f minor (4♭)" />
            <MenuItem text="E♭ Major / c minor (3♭)" />
            <MenuItem text="B♭ Major / g minor (2♭)" />
            <MenuItem text="F Major / d minor (♭)" />
            <MenuItem text="C Major / a minor" />
            <MenuItem text="G Major / e minor (♯)" />
            <MenuItem text="D Major / b minor (2♯)" />
            <MenuItem text="A Major / f♯ minor (3♯)" />
            <MenuItem text="E Major / c♯ minor (4♯)" />
            <MenuItem text="B Major / g♯ minor (5♯)" />
            <MenuItem text="F♯ Major / d♯ minor (6♯)" />
            <MenuDivider />
            <MenuItem text="No change" />
          </MenuItem>
          <MenuDivider />
          <MenuItem text="Edit Barline">
            <MenuItem text="Single" />
            <MenuItem text="Double" />
            <MenuItem text="Final" />
            <MenuDivider />
            <MenuItem text="Start Repeat" />
            <MenuItem text="End Repeat" />
          </MenuItem>
          <MenuItem text="Insert Bar" />
          <MenuItem text="Remove Next Bar" />
        </Menu>
      }
      interactionKind="hover"
      minimal={true}
    >
      {children}
    </Popover>
  );
}
