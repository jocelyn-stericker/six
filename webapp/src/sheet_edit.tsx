import React, { useState } from "react";
import Sheet, { Barline } from "./sheet";
import { Action, State } from "./store";

const BetweenBarPopover = React.lazy(() => import("./between_bar_popover"));

interface Props {
  tool: "notes" | "bars" | "select";
  appState: State;
  dispatch: (action: Action) => void;
}

function SheetEdit({ tool, appState, dispatch }: Props) {
  const [hoverElement, setHoverElementChanged] = useState<{
    id: number;
    kind: number;
    bbox: [number, number, number, number];
  } | null>(null);
  const [hoverTime, setHoverTime] = useState<[number, number, number] | null>(
    null
  );

  const [numChanges, setNumChanges] = useState(0);

  const hoverMatchesAny =
    hoverTime &&
    appState.song.part.bars.some(
      (bar, barIdx) =>
        barIdx === hoverTime[0] &&
        bar.notes.some(
          note =>
            note.startNum === hoverTime[1] && note.startDen === hoverTime[2]
        )
    );

  return (
    <div>
      <Sheet
        hoverElement={hoverElement ? hoverElement.id : null}
        onHoverElementChanged={setHoverElementChanged}
        hoverTime={hoverTime}
        onHoverTimeChanged={time => setHoverTime(time)}
        onClick={(time, mode) => {
          if (!time || tool !== "notes") {
            return;
          }

          if (mode === "rnc") {
            dispatch({
              type: "ADD_NOTE",
              bar: time[0],
              num: time[1],
              den: time[2],
              duration: -3,
              dots: 0
            });
            setNumChanges(numChanges + 1);
          }
        }}
      >
        <song
          freezeSpacing={hoverTime == null ? undefined : numChanges}
          key={`${appState.song.global.tsNum}_${appState.song.global.tsDen}`}
        >
          <staff>
            <between
              boundingClassName="six-between"
              clef={true}
              tsNum={appState.song.global.tsNum}
              tsDen={appState.song.global.tsDen}
            />
            {appState.song.part.bars.map((bar, barIdx) => (
              <React.Fragment key={barIdx}>
                <bar
                  numer={appState.song.global.tsNum}
                  denom={appState.song.global.tsDen}
                  boundingClassName={
                    tool === "notes" &&
                    hoverTime &&
                    hoverTime[0] === barIdx &&
                    "six-bar-hover-bg"
                  }
                  className={
                    tool === "notes" &&
                    hoverTime &&
                    hoverTime[0] === barIdx &&
                    "six-bar-hover"
                  }
                >
                  {bar.notes.map(
                    ({ dots, duration, startNum, startDen }, idx) => (
                      <rnc
                        className="six-real-note"
                        boundingClassName="six-real-note-bg"
                        key={idx}
                        noteValue={duration}
                        dots={dots}
                        startNum={startNum}
                        startDen={startDen}
                        isNote={true}
                      />
                    )
                  )}
                  {tool === "notes" &&
                    !hoverMatchesAny &&
                    hoverTime &&
                    hoverTime[0] === barIdx && (
                      <rnc
                        boundingClassName="six-note-to-add-bg"
                        className="six-note-to-add"
                        noteValue={-3}
                        dots={0}
                        startNum={hoverTime[1]}
                        startDen={hoverTime[2]}
                        isNote={true}
                      />
                    )}
                </bar>
                <between
                  barline={
                    bar.barline === "normal" ? Barline.Normal : Barline.Final
                  }
                />
              </React.Fragment>
            ))}
          </staff>
        </song>
      </Sheet>

      {tool === "bars" && hoverElement && hoverElement.kind === 1 && (
        <React.Suspense fallback={null}>
          <div
            style={{
              position: "absolute",
              top: hoverElement.bbox[1],
              left: hoverElement.bbox[0],
              width: hoverElement.bbox[2] - hoverElement.bbox[0] + 20,
              height: hoverElement.bbox[3] - hoverElement.bbox[1] - 20,
              margin: 0,
              padding: 0
            }}
          >
            <BetweenBarPopover
              tsNum={appState.song.global.tsNum}
              tsDen={appState.song.global.tsDen}
              setTs={([num, den]) =>
                dispatch({
                  type: "SET_TS",
                  num,
                  den,
                  prevNum: appState.song.global.tsNum,
                  prevDen: appState.song.global.tsDen
                })
              }
            >
              <div
                style={{
                  width: hoverElement.bbox[2] - hoverElement.bbox[0] + 10,

                  height: hoverElement.bbox[3] - hoverElement.bbox[1],
                  margin: 0,
                  padding: 0,
                  cursor: "pointer"
                }}
              />
            </BetweenBarPopover>
          </div>
        </React.Suspense>
      )}
    </div>
  );
}

export default React.memo(SheetEdit);
