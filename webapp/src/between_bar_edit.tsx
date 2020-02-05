import React from "react";

import {
  Dialog,
  Classes,
  Button,
  ButtonGroup,
  Checkbox
} from "@blueprintjs/core";

export interface Props {
  onClose: () => void;
  tsNum: number;
  tsDen: number;
  setTs: (ts: [number, number]) => void;
}

export default function BetweenBarEdit({
  onClose,
  tsNum,
  tsDen,
  setTs
}: Props) {
  return (
    <Dialog isOpen={true} onClose={onClose} title="Edit Barline / Signatures">
      <div className={Classes.DIALOG_BODY}>
        <Checkbox label="Set Time Signature" checked={true} disabled={true} />
        <div style={{ display: "block", marginBottom: 4 }}>
          Simple:{" "}
          <ButtonGroup large={true} style={{ marginRight: 4 }}>
            <Button
              onClick={() => setTs([4, 4])}
              active={tsNum === 4 && tsDen === 4}
            >
              4/4
            </Button>
            <Button
              onClick={() => setTs([2, 2])}
              active={tsNum === 2 && tsDen === 2}
            >
              2/2
            </Button>
          </ButtonGroup>
          <ButtonGroup large={true} style={{ marginRight: 4 }}>
            <Button
              onClick={() => setTs([2, 4])}
              active={tsNum === 2 && tsDen === 4}
            >
              2/4
            </Button>
            <Button
              onClick={() => setTs([4, 8])}
              active={tsNum === 4 && tsDen === 8}
            >
              4/8
            </Button>
          </ButtonGroup>
          <ButtonGroup large={true} style={{ marginRight: 4 }}>
            <Button
              onClick={() => setTs([3, 4])}
              active={tsNum === 3 && tsDen === 4}
            >
              3/4
            </Button>
            <Button
              onClick={() => setTs([3, 8])}
              active={tsNum === 3 && tsDen === 8}
            >
              3/8
            </Button>
          </ButtonGroup>
        </div>

        <div style={{ display: "block", marginBottom: 4 }}>
          Compound:{" "}
          <ButtonGroup large={true} style={{ marginRight: 4 }}>
            <Button
              onClick={() => setTs([6, 8])}
              active={tsNum === 6 && tsDen === 8}
            >
              6/8
            </Button>
            <Button
              onClick={() => setTs([6, 4])}
              active={tsNum === 6 && tsDen === 4}
            >
              6/4
            </Button>
            <Button
              onClick={() => setTs([6, 16])}
              active={tsNum === 6 && tsDen === 16}
            >
              6/16
            </Button>
          </ButtonGroup>
          <ButtonGroup large={true} style={{ marginRight: 4 }}>
            <Button
              onClick={() => setTs([9, 8])}
              active={tsNum === 9 && tsDen === 8}
            >
              9/8
            </Button>
            <Button
              onClick={() => setTs([12, 8])}
              active={tsNum === 12 && tsDen === 8}
            >
              12/8
            </Button>
          </ButtonGroup>
        </div>
      </div>
    </Dialog>
  );
}
