import React, { createRef, useMemo, useRef, useState } from "react";

import Sheet from "./sheet";
import { Render } from "./sheet/reconciler";
import { Action, addNote, removeNote, State } from "./store";
import splitDurationIntoParts, {
  NoteAddPatch,
} from "./split_duration_into_parts";
import Between from "./between";

const NotePopover = React.lazy(() => import("./note_popover"));
const NoteInsertMenu = React.lazy(() => import("./note_insert_menu"));

interface Props {
  appState: State;
  dispatch: (action: Action) => void;
}

function SheetEdit({ appState, dispatch }: Props) {
  const songRef = useRef<Render>(null);
  const barRefs = useMemo(
    () =>
      Array.from({ length: appState.song.part.bars.length }).map(() =>
        createRef<number>(),
      ),
    [appState.song.part.bars.length],
  );

  const [preview, setPreview] = useState<NoteAddPatch | null>(null);
  const [numChanges, setNumChanges] = useState(0);
  const [noteMutationClickPos, setNoteMutationClickPos] = useState<
    [number, number] | null
  >(null);

  const hoverMatchesAny = false;

  let currTs = appState.song.global.between[0].ts;

  return (
    <div
      style={{ position: "relative" }}
      onMouseOut={() => {
        if (!noteMutationClickPos) {
          setPreview(null);
        }
      }}
    >
      {noteMutationClickPos && preview && (
        <React.Suspense fallback={null}>
          <NoteInsertMenu
            pos={noteMutationClickPos}
            pitchModifier={preview.pitch.modifier ?? null}
            onClose={() => {
              setPreview(null);
              setNoteMutationClickPos(null);
            }}
            onAddNote={(duration, modifier) => {
              let insertion = splitDurationIntoParts(
                songRef.current,
                appState,
                barRefs[preview.barIdx].current,
                preview.barIdx,
                [preview.startTime[0], preview.startTime[1]],
                duration,
                preview.pitch,
              );
              setPreview(null);
              if (insertion) {
                dispatch(
                  addNote({
                    barIdx: insertion.barIdx,
                    startTime: insertion.startTime,
                    divisions: insertion.divisions,
                    pitch: {
                      base: preview.pitch.base,
                      modifier,
                    },
                  }),
                );
              }
              setNoteMutationClickPos(null);
              setNumChanges(numChanges + 1);
            }}
          />
        </React.Suspense>
      )}
      <Sheet
        onHover={hoverInfo => {
          if (noteMutationClickPos) {
            return;
          }
          if (
            hoverInfo.bar == null ||
            hoverInfo.time == null ||
            hoverInfo.pitch == null
          ) {
            setPreview(null);
            return;
          }
          setPreview(
            splitDurationIntoParts(
              songRef.current,
              appState,
              barRefs[hoverInfo.bar].current,
              hoverInfo.bar,
              hoverInfo.time,
              [1, 4],
              hoverInfo.pitch,
            ),
          );
        }}
        onMouseDown={(_, ev) => {
          if (preview) {
            setNoteMutationClickPos([ev.clientX, ev.clientY]);
          }
        }}
      >
        <song
          freezeSpacing={preview == null ? undefined : numChanges}
          ref={songRef}
          width={215.9}
          height={279.4}
          title={appState.song.global.title}
          author={appState.song.global.author}
        >
          <staff>
            <Between
              appState={appState}
              dispatch={ev => {
                setNumChanges(numChanges + 1);
                dispatch(ev);
              }}
              beforeBar={0}
            />
            {appState.song.part.bars.map((bar, barIdx) => {
              currTs = appState.song.global.between[barIdx]?.ts ?? currTs;

              // TODO: have stable keys even when adding/removing bars.
              return (
                <React.Fragment key={`${currTs[0]}_${currTs[1]}_${barIdx}`}>
                  <bar
                    ref={barRefs[barIdx]}
                    numer={currTs[0]}
                    denom={currTs[1]}
                    className={
                      preview && preview.barIdx === barIdx
                        ? "six-bar-hover"
                        : "six-bar"
                    }
                  >
                    {bar.notes.map(
                      (
                        { divisions, startTime: tiedStartTime, pitch },
                        divisionIdx,
                      ) => (
                        <React.Fragment key={divisionIdx}>
                          {divisions.map(
                            ({ noteValue, dots, startTime }, jdx) => (
                              <rnc
                                className="six-real-note"
                                key={jdx}
                                noteValue={noteValue}
                                dots={dots}
                                startNum={startTime[0]}
                                startDen={startTime[1]}
                                isNote={true}
                                isTemporary={false}
                                pitch={pitch.base}
                                pitchModifier={pitch.modifier}
                                html={({ width, height }) => (
                                  <React.Suspense fallback={null}>
                                    <NotePopover
                                      onDeleteNote={() => {
                                        setNumChanges(numChanges + 1);
                                        dispatch(
                                          removeNote({
                                            barIdx,
                                            startTime: tiedStartTime,
                                            divisions,
                                            pitch,
                                          }),
                                        );
                                      }}
                                    >
                                      <div
                                        onMouseOver={() => setPreview(null)}
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
                      preview &&
                      preview.barIdx === barIdx &&
                      preview.divisions.map((div, idx) => (
                        <rnc
                          key={idx}
                          className="six-note-to-add"
                          noteValue={div.noteValue}
                          dots={div.dots}
                          startNum={div.startTime[0]}
                          startDen={div.startTime[1]}
                          isNote={true}
                          isTemporary={true}
                          pitch={preview.pitch.base}
                          pitchModifier={preview.pitch.modifier}
                        />
                      ))}
                  </bar>
                  <Between
                    appState={appState}
                    dispatch={ev => {
                      setNumChanges(numChanges + 1);
                      dispatch(ev);
                    }}
                    beforeBar={barIdx + 1}
                  />
                </React.Fragment>
              );
            })}
          </staff>
        </song>
      </Sheet>
    </div>
  );
}

export default React.memo(SheetEdit);
