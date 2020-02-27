import React, { useState, useRef, useMemo, createRef } from "react";
import PieMenu, { Slice } from "react-pie-menu";
import { ThemeProvider, css } from "styled-components";

import Sheet, { Barline } from "./sheet";
import { Render, Clef } from "./sheet/reconciler";
import { Action, State, TiedNote, Clef as ClefStr } from "./store";

const BetweenBarPopover = React.lazy(() => import("./between_bar_popover"));
const NotePopover = React.lazy(() => import("./note_popover"));
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

interface Props {
  appState: State;
  dispatch: (action: Action) => void;
}

interface ProposedInsertion {
  barIdx: number;
  startNum: number;
  startDen: number;
  divisions: TiedNote;
}

function count(noteValue: number, dots: number) {
  let base = Math.pow(2, noteValue);
  let total = base;
  for (let i = 0; i < dots; ++i) {
    total += base / 2;
    base / 2;
  }

  return total;
}

function getProposedInsertion(
  render: Render | null,
  appState: State,
  barEntity: number | null,
  time: [number, number, number] | null,
  insertionDuration: Array<number>,
): ProposedInsertion | null {
  if (!render) {
    return null;
  }
  if (!time) {
    return null;
  }
  if (!barEntity) {
    return null;
  }

  const rawDivisions = render.split_note(
    barEntity,
    time[1],
    time[2],
    insertionDuration[0],
    insertionDuration[1],
  );
  const start = time[1] / time[2];

  const divisions = [];
  let end = start;
  for (let i = 0; i < rawDivisions.length; i += 4) {
    end += count(rawDivisions[i], rawDivisions[i + 1]);
    divisions.push({
      noteValue: rawDivisions[i],
      dots: rawDivisions[i + 1],
      startNum: rawDivisions[i + 2],
      startDen: rawDivisions[i + 3],
    });
  }
  if (!divisions.length) {
    return null;
  }

  if (
    appState.song.part.bars[time[0]].notes.some(note => {
      let noteStart = note.startNum / note.startDen;
      let noteEnd =
        noteStart +
        note.divisions.reduce(
          (sum, { noteValue, dots }) => sum + count(noteValue, dots),
          0,
        );
      // TODO: check if this note is in the middle of the proposed one.
      return (
        (start <= noteStart && end > noteStart) ||
        (start < noteEnd && end >= noteEnd)
      );
    })
  ) {
    return null;
  }

  return {
    barIdx: time[0],
    startNum: time[1],
    startDen: time[2],
    divisions,
  };
}

function clefStrToNum(clef: ClefStr): Clef {
  if (clef === "g") {
    return Clef.G;
  }
  if (clef === "f") {
    return Clef.F;
  }
  if (clef === "percussion") {
    return Clef.Percussion;
  }

  throw new Error("Unexpected clef");
}

function SheetEdit({ appState, dispatch }: Props) {
  const [insertionDuration, setInsertionDuration] = useState([1, 8]);
  const [
    proposedInsertion,
    setProposedInsertion,
  ] = useState<ProposedInsertion | null>(null);
  const [editMode, setEditMode] = useState(false);

  const songRef = useRef<Render>(null);

  const [numChanges, setNumChanges] = useState(0);
  const [dragState, setDragState] = useState<{
    startX: number;
    startY: number;
  } | null>(null);

  const barRefs = useMemo(
    () =>
      Array.from({ length: appState.song.part.bars.length }).map(() =>
        createRef<number>(),
      ),
    [appState.song.part.bars.length],
  );

  const hoverMatchesAny = false;

  function addNote(frac: [number, number]) {
    if (proposedInsertion) {
      let newProposedInsertion = getProposedInsertion(
        songRef.current,
        appState,
        barRefs[proposedInsertion.barIdx].current,
        [
          proposedInsertion.barIdx,
          proposedInsertion.startNum,
          proposedInsertion.startDen,
        ],
        frac,
      );
      if (newProposedInsertion) {
        dispatch({
          type: "ADD_NOTE",
          barIdx: newProposedInsertion.barIdx,
          startNum: newProposedInsertion.startNum,
          startDen: newProposedInsertion.startDen,
          divisions: newProposedInsertion.divisions,
        });
      }
      setDragState(null);
      setNumChanges(numChanges + 1);
    }
  }

  return (
    <div style={{ position: "relative" }}>
      {dragState && (
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
              centerX={`${dragState.startX}px`}
              centerY={`${dragState.startY}px`}
              centerRadius="20px"
              radius="100px"
            >
              <Slice onSelect={() => addNote([1, 1])}>1</Slice>
              <Slice onSelect={() => addNote([1, 2])}>1/2</Slice>
              <Slice onSelect={() => addNote([1, 4])}>1/4</Slice>
              <Slice onSelect={() => addNote([1, 8])}>1/8</Slice>
              <Slice onSelect={() => addNote([1, 16])}>1/16</Slice>
              <Slice>.</Slice>
              <Slice>#</Slice>
              <Slice>b</Slice>
            </PieMenu>
          </div>
        </ThemeProvider>
      )}
      <Sheet
        onHoverTimeChanged={time => {
          if (dragState) {
            return;
          }
          if (!time) {
            setProposedInsertion(null);
            return;
          }
          setProposedInsertion(
            getProposedInsertion(
              songRef.current,
              appState,
              barRefs[time[0]].current,
              time,
              insertionDuration,
            ),
          );
        }}
        onMouseMove={_ev => {
          if (editMode) {
            setTimeout(() => setEditMode(false), 0);
          }
        }}
        onMouseDown={(_, ev) => {
          if (proposedInsertion) {
            setDragState({
              startX: ev.clientX,
              startY: ev.clientY,
            });
          }
        }}
        onMouseUp={() => {
          if (editMode) {
            return;
          }
          setDragState(null);
          if (insertionDuration[0] / insertionDuration[1] > 1 / 4) {
            setInsertionDuration([1, 4]);
          }

          if (proposedInsertion) {
            dispatch({
              type: "ADD_NOTE",
              barIdx: proposedInsertion.barIdx,
              startNum: proposedInsertion.startNum,
              startDen: proposedInsertion.startDen,
              divisions: proposedInsertion.divisions,
            });
            setNumChanges(numChanges + 1);
          }
        }}
      >
        <song
          freezeSpacing={proposedInsertion == null ? undefined : numChanges}
          key={`${appState.song.global.tsNum}_${appState.song.global.tsDen}`}
          ref={songRef}
          width={215.9}
          height={279.4}
          title={appState.song.global.title}
          author={appState.song.global.author}
        >
          <staff>
            <between
              clef={clefStrToNum(appState.song.global.clef)}
              tsNum={appState.song.global.tsNum}
              tsDen={appState.song.global.tsDen}
              ks={appState.song.global.ks}
              className="between-bars"
              html={({ width, height }) => (
                <React.Suspense fallback={null}>
                  <BetweenBarPopover
                    tsNum={appState.song.global.tsNum}
                    tsDen={appState.song.global.tsDen}
                    setClef={clef =>
                      dispatch({
                        type: "SET_CLEF",
                        clef,
                        prevClef: appState.song.global.clef,
                      })
                    }
                    setKs={ks =>
                      dispatch({
                        type: "SET_KS",
                        ks,
                        prevKs: appState.song.global.ks,
                      })
                    }
                    setTs={([num, den]) =>
                      dispatch({
                        type: "SET_TS",
                        num,
                        den,
                        prevNum: appState.song.global.tsNum,
                        prevDen: appState.song.global.tsDen,
                      })
                    }
                    onInsertBarRight={() => {
                      dispatch({
                        type: "ADD_BAR",
                        barIdx: 0,
                        bar: {
                          barline: "normal",
                          notes: [],
                        },
                      });
                    }}
                    onRemoveBarRight={
                      appState.song.part.bars[0] &&
                      (() => {
                        dispatch({
                          type: "REMOVE_BAR",
                          barIdx: 0,
                          bar: appState.song.part.bars[0],
                        });
                      })
                    }
                  >
                    <div
                      onMouseOver={() => setEditMode(true)}
                      className="between-edit"
                      style={{
                        width,
                        height,
                        cursor: "pointer",
                      }}
                    />
                  </BetweenBarPopover>
                </React.Suspense>
              )}
            />
            {appState.song.part.bars.map((bar, barIdx) => (
              <React.Fragment key={barIdx}>
                <bar
                  ref={barRefs[barIdx]}
                  numer={appState.song.global.tsNum}
                  denom={appState.song.global.tsDen}
                  className={
                    !editMode &&
                    proposedInsertion &&
                    proposedInsertion.barIdx === barIdx
                      ? "six-bar-hover"
                      : "six-bar"
                  }
                >
                  {bar.notes.map(
                    (
                      {
                        divisions,
                        startNum: tiedStartNum,
                        startDen: tiedStartDen,
                      },
                      divisionIdx,
                    ) => (
                      <React.Fragment key={divisionIdx}>
                        {divisions.map(
                          ({ noteValue, dots, startNum, startDen }, jdx) => (
                            <rnc
                              className="six-real-note"
                              key={jdx}
                              noteValue={noteValue}
                              dots={dots}
                              startNum={startNum}
                              startDen={startDen}
                              isNote={true}
                              isTemporary={false}
                              html={({ width, height }) => (
                                <React.Suspense fallback={null}>
                                  <NotePopover
                                    onDeleteNote={() => {
                                      dispatch({
                                        type: "REMOVE_NOTE",
                                        barIdx,
                                        startNum: tiedStartNum,
                                        startDen: tiedStartDen,
                                        divisions,
                                      });
                                    }}
                                  >
                                    <div
                                      onMouseOver={() =>
                                        setProposedInsertion(null)
                                      }
                                      style={{
                                        width,
                                        height,
                                        cursor: "pointer",
                                      }}
                                    />
                                  </NotePopover>
                                </React.Suspense>
                              )}
                            />
                          ),
                        )}
                      </React.Fragment>
                    ),
                  )}
                  {!hoverMatchesAny &&
                    !editMode &&
                    proposedInsertion &&
                    proposedInsertion.barIdx === barIdx &&
                    proposedInsertion.divisions.map((div, idx) => (
                      <rnc
                        key={idx}
                        className="six-note-to-add"
                        noteValue={div.noteValue}
                        dots={div.dots}
                        startNum={div.startNum}
                        startDen={div.startDen}
                        isNote={true}
                        isTemporary={true}
                      />
                    ))}
                </bar>
                <between
                  barline={
                    bar.barline === "normal" ? Barline.Normal : Barline.Final
                  }
                  className="between-bars"
                  html={({ width, height }) => (
                    <React.Suspense fallback={null}>
                      <BetweenBarPopover
                        tsNum={appState.song.global.tsNum}
                        tsDen={appState.song.global.tsDen}
                        setClef={clef =>
                          dispatch({
                            type: "SET_CLEF",
                            clef,
                            prevClef: appState.song.global.clef,
                          })
                        }
                        setKs={ks =>
                          dispatch({
                            type: "SET_KS",
                            ks,
                            prevKs: appState.song.global.ks,
                          })
                        }
                        setTs={([num, den]) =>
                          dispatch({
                            type: "SET_TS",
                            num,
                            den,
                            prevNum: appState.song.global.tsNum,
                            prevDen: appState.song.global.tsDen,
                          })
                        }
                        onInsertBarRight={() => {
                          dispatch({
                            type: "ADD_BAR",
                            barIdx: barIdx + 1,
                            bar: {
                              barline: "normal",
                              notes: [],
                            },
                          });
                        }}
                        onRemoveBarRight={
                          appState.song.part.bars[barIdx + 1] &&
                          (() => {
                            dispatch({
                              type: "REMOVE_BAR",
                              barIdx: barIdx + 1,
                              bar: appState.song.part.bars[barIdx + 1],
                            });
                          })
                        }
                      >
                        <div
                          onMouseOver={() => setEditMode(true)}
                          className="between-edit"
                          style={{
                            width,
                            height,
                            cursor: "pointer",
                          }}
                        />
                      </BetweenBarPopover>
                    </React.Suspense>
                  )}
                />
              </React.Fragment>
            ))}
          </staff>
        </song>
      </Sheet>
    </div>
  );
}

export default React.memo(SheetEdit);
