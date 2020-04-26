import React from "react";
import {
  Button,
  Navbar,
  NavbarDivider,
  NavbarGroup,
  NavbarHeading,
  Tooltip,
} from "@blueprintjs/core";

export interface Props {
  onSave: () => void;
  onOpen: () => void;
  onTrash: () => void;
}

export default function SixNavbar(props: Props) {
  return (
    <Navbar style={{ position: "sticky", top: 0, left: 0 }}>
      <NavbarGroup>
        <NavbarHeading>six-eight.app</NavbarHeading>
        <NavbarDivider />
        <Tooltip content="Save as Six Eight PDF&hellip;" position="bottom">
          <Button icon="floppy-disk" onClick={() => props.onSave()} />
        </Tooltip>
        <div style={{ width: 8 }} />
        <Tooltip content="Open Six Eight PDF&hellip;" position="bottom">
          <Button icon="folder-open" onClick={() => props.onOpen()} />
        </Tooltip>
        <div style={{ width: 8 }} />
        <Tooltip content="Reset sheet" position="bottom">
          <Button
            icon="trash"
            onClick={() => {
              if (
                confirm(
                  "This will clear the whole document, and cannot be undone. Make sure you have saved any work you want to keep. Are you sure?",
                )
              ) {
                props.onTrash();
              }
            }}
          />
        </Tooltip>
      </NavbarGroup>
    </Navbar>
  );
}
