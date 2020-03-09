import React from "react";
import PieMenu, { Slice } from "react-pie-menu";

import { ThemeProvider, css } from "styled-components";

const theme = {
  slice: {
    container: css`
      background: ${({ centerRadius }: any) =>
        `radial-gradient(transparent ${centerRadius}, #004643cc ${centerRadius})`};
      color: #abd1c6;
      :hover {
        background: ${({ centerRadius }: any) =>
          `radial-gradient(transparent ${centerRadius}, #f9bc60 ${centerRadius})`};
        color: #001e1d;
      }
    `,
  },
};

export default function NoteInsertMenu({
  pos,
  onAddNote,
}: {
  pos: [number, number];
  onAddNote: (duration: [number, number]) => void;
}) {
  return (
    <ThemeProvider theme={theme}>
      <div
        style={{
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          zIndex: 100,
          userSelect: "none",
        }}
      >
        <PieMenu
          centerX={`${pos[0]}px`}
          centerY={`${pos[1]}px`}
          centerRadius="20px"
          radius="100px"
        >
          <Slice onSelect={() => onAddNote([1, 1])}>1</Slice>
          <Slice onSelect={() => onAddNote([1, 2])}>1/2</Slice>
          <Slice onSelect={() => onAddNote([1, 4])}>1/4</Slice>
          <Slice onSelect={() => onAddNote([1, 8])}>1/8</Slice>
          <Slice onSelect={() => onAddNote([1, 16])}>1/16</Slice>
          <Slice>.</Slice>
          <Slice>#</Slice>
          <Slice>b</Slice>
        </PieMenu>
      </div>
    </ThemeProvider>
  );
}
