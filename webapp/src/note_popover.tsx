import React from "react";
import { Popover, Menu, MenuItem } from "@blueprintjs/core";

export interface Props {
  children: any;
  onDeleteNote: () => void;
}

export default function NotePopover({ children, onDeleteNote }: Props) {
  return (
    <Popover
      position="right-top"
      popoverClassName="bp3-dark"
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
            text="Delete Note"
            icon="delete"
            onClick={onDeleteNote}
            popoverProps={{
              interactionKind: "hover",
            }}
          />
        </Menu>
      }
    >
      {children}
    </Popover>
  );
}
