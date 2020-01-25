import React from "react";

import {
  Popover,
  Tooltip,
  Card,
  Button,
  ButtonGroup,
  Navbar,
  Radio,
  RadioGroup,
  Checkbox
} from "@blueprintjs/core";

export type Tool = "notes" | "bars" | "select";

export interface Props {
  tool: Tool;
  canUndo: boolean;
  sixteenth: boolean;
  tuplets: boolean;

  onSetTool: (tool: Tool) => void;
  onSave: () => void;
  onOpen: () => void;
  onUndo: () => void;
  onSixteenthChanged: (on: boolean) => void;
  onTupletChanged: (on: boolean) => void;
}

export default function Toolbar(props: Props) {
  return (
    <React.Fragment>
      <Navbar className="six-navbar">
        <Navbar.Group align="left">
          <Navbar.Heading className="six-heading">Six Eight</Navbar.Heading>
          <ButtonGroup>
            <Tooltip position="bottom" usePortal={false}>
              <Button icon="floppy-disk" onClick={props.onSave} />
              <React.Fragment>Save as PDF&hellip;</React.Fragment>
            </Tooltip>
            <Tooltip position="bottom" usePortal={false}>
              <Button icon="document-open" onClick={props.onOpen} />
              <React.Fragment>Open Six Eight PDF&hellip;</React.Fragment>
            </Tooltip>
            <Tooltip position="bottom" usePortal={false}>
              <Button
                icon="undo"
                disabled={!props.canUndo}
                onClick={props.onUndo}
              />
              <React.Fragment>Undo</React.Fragment>
            </Tooltip>
          </ButtonGroup>
          <Navbar.Divider />
        </Navbar.Group>
        <Navbar.Group>
          <RadioGroup
            inline={true}
            selectedValue={props.tool}
            onChange={ev => props.onSetTool(ev.currentTarget.value as any)}
          >
            <Radio className="six-navbar-control" value="notes">
              Add Notes (
              <span className="six-navbar-settings">
                <Popover minimal={true}>
                  <Tooltip position="bottom">
                    <Button
                      icon="settings"
                      minimal={true}
                      intent="primary"
                      small={true}
                      active={props.sixteenth || props.tuplets}
                      disabled={props.tool !== "notes"}
                    />
                    <>Insertion Options</>
                  </Tooltip>
                  <Card>
                    <Checkbox
                      label="Sixteenth or shorter"
                      checked={props.sixteenth}
                      onChange={ev =>
                        props.onSixteenthChanged(ev.currentTarget.checked)
                      }
                    />
                    <Checkbox
                      label="Tuplets"
                      checked={props.tuplets}
                      onChange={ev =>
                        props.onTupletChanged(ev.currentTarget.checked)
                      }
                    />
                  </Card>
                </Popover>
              </span>
              )
            </Radio>
            <Radio className="six-navbar-control" value="select">
              <Tooltip position="bottom">
                <>Select Notes</>
                <>Add dynamics/articulation to notes, remove notes, &hellip;</>
              </Tooltip>
            </Radio>
            <Tooltip position="bottom">
              <Radio
                className="six-navbar-control"
                label="Edit Bars & Signatures"
                value="bars"
              />
              <>
                Change clefs/signatures/instructions/phrasing, add/remove
                bars/repeats, ...
              </>
            </Tooltip>
          </RadioGroup>
        </Navbar.Group>
      </Navbar>
    </React.Fragment>
  );
}
