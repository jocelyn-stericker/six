import React, { useState, useRef, useMemo, createRef } from "react";

import Sheet, { Barline } from "./sheet";
import { Render, Clef } from "./sheet/reconciler";
import { Action, State, TiedNote, Clef as ClefStr } from "./store";

const BetweenBarPopover = React.lazy(() => import("./between_bar_popover"));
const NotePopover = React.lazy(() => import("./note_popover"));

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

const STEPS = [
  [1, 1],
  [3, 4],
  [1, 2],
  [3, 8],
  [1, 4],
  [3, 16],
  [1, 8],
  [3, 32],
  [1, 16],
];

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

  const songRef = useRef<Render>(null);

  const [numChanges, setNumChanges] = useState(0);
  const [dragState, setDragState] = useState<{
    startX: number;
    startY: number;
    origInsertionDuration: [number, number];
  } | null>(null);

  const barRefs = useMemo(
    () =>
      Array.from({ length: appState.song.part.bars.length }).map(() =>
        createRef<number>(),
      ),
    [appState.song.part.bars.length],
  );

  const hoverMatchesAny = false;

  return (
    <div style={{ position: "relative" }}>
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
        onMouseMove={ev => {
          if (dragState && proposedInsertion) {
            let deltaX = ev.clientX - dragState.startX;
            const steps = Math.trunc(deltaX / 30);
            let step = STEPS.findIndex(
              x =>
                x[0] === dragState.origInsertionDuration[0] &&
                x[1] === dragState.origInsertionDuration[1],
            );
            step = Math.min(Math.max(step - steps, 0), STEPS.length - 1);
            const frac = STEPS[step];
            if (
              frac[1] !== insertionDuration[1] ||
              frac[0] !== insertionDuration[0]
            ) {
              setInsertionDuration(frac);
              setProposedInsertion(
                getProposedInsertion(
                  songRef.current,
                  appState,
                  barRefs[proposedInsertion.barIdx].current,
                  [
                    proposedInsertion.barIdx,
                    proposedInsertion.startNum,
                    proposedInsertion.startDen,
                  ],
                  frac,
                ),
              );
            }
          }
        }}
        onMouseDown={(_, ev) => {
          setDragState({
            startX: ev.clientX,
            startY: ev.clientY,
            origInsertionDuration: [insertionDuration[0], insertionDuration[1]],
          });
        }}
        onMouseUp={() => {
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
                    setTs={([num, den]) =>
                      dispatch({
                        type: "SET_TS",
                        num,
                        den,
                        prevNum: appState.song.global.tsNum,
                        prevDen: appState.song.global.tsDen,
                      })
                    }
                  >
                    <div
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
                    proposedInsertion && proposedInsertion.barIdx === barIdx
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
                        setTs={([num, den]) =>
                          dispatch({
                            type: "SET_TS",
                            num,
                            den,
                            prevNum: appState.song.global.tsNum,
                            prevDen: appState.song.global.tsDen,
                          })
                        }
                      >
                        <div
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
