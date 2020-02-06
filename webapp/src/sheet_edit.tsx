import React, { useState } from "react";
import Sheet, { Barline } from "./sheet";
import { Action, State } from "./store";

const BetweenBarPopover = React.lazy(() => import("./between_bar_popover"));
const BetweenBarEdit = React.lazy(() => import("./between_bar_edit"));

interface Props {
  appState: State;
  dispatch: (action: Action) => void;
}

function SheetEdit({ appState, dispatch }: Props) {
  const [showBetweenBarEdit, setShowBetweenBarEdit] = useState(false);
  const [hoverElement, setHoverElementChanged] = useState<{
    id: number;
    kind: number;
    bbox: [number, number, number, number];
  } | null>(null);
  const [hoverTime, setHoverTime] = useState<[number, number, number] | null>(
    null
  );

  return (
    <React.Fragment>
      <Sheet
        hoverElement={hoverElement ? hoverElement.id : null}
        onHoverElementChanged={setHoverElementChanged}
        hoverTime={hoverTime}
        onHoverTimeChanged={time => setHoverTime(time)}
        onClick={(time, mode) => {
          if (!time) {
            return;
          }

          if (mode === "between-bars") {
            setShowBetweenBarEdit(true);
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
          }
        }}
      >
        <song
          freezeSpacing={hoverTime == null ? undefined : hoverTime[0]}
          key={`${appState.song.global.tsNum}_${appState.song.global.tsDen}`}
        >
          <staff>
            <between
              clef={true}
              tsNum={appState.song.global.tsNum}
              tsDen={appState.song.global.tsDen}
            />
            {appState.song.part.bars.map((bar, barIdx) => (
              <React.Fragment key={barIdx}>
                <bar
                  numer={appState.song.global.tsNum}
                  denom={appState.song.global.tsDen}
                >
                  {bar.notes.map(
                    ({ dots, duration, startNum, startDen }, idx) =>
                      (!hoverTime ||
                        hoverTime[0] !== barIdx ||
                        hoverTime[1] !== startNum ||
                        hoverTime[2] !== startDen) && (
                        <rnc
                          key={idx}
                          noteValue={duration}
                          dots={dots}
                          startNum={startNum}
                          startDen={startDen}
                          isNote={true}
                        />
                      )
                  )}
                  {hoverTime && hoverTime[0] === barIdx && (
                    <rnc
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
      {hoverElement && hoverElement.kind === 1 && (
        <div
          className="six-note-editor-active-target-bg"
          style={{
            position: "absolute",
            top: hoverElement.bbox[1],
            left: hoverElement.bbox[0],
            width: hoverElement.bbox[2] - hoverElement.bbox[0] + 10,
            height: hoverElement.bbox[3] - hoverElement.bbox[1],
            margin: 0,
            padding: 0
          }}
        />
      )}
      {hoverElement && hoverElement.kind === 1 && (
        <React.Suspense fallback={null}>
          <div
            style={{
              position: "absolute",
              top: hoverElement.bbox[1],
              left: hoverElement.bbox[0],
              width: hoverElement.bbox[2] - hoverElement.bbox[0] + 10,
              height: hoverElement.bbox[3] - hoverElement.bbox[1],
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
      {showBetweenBarEdit && (
        <React.Suspense fallback={null}>
          <BetweenBarEdit
            onClose={() => setShowBetweenBarEdit(false)}
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
          />
        </React.Suspense>
      )}
    </React.Fragment>
  );
}

export default React.memo(SheetEdit);
