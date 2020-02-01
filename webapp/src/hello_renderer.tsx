import React, { useState } from "react";
import SheetMusicView, { Barline } from "./sheet_music_view";

const BetweenBarEdit = React.lazy(() => import("./between_bar_edit"));

interface Note {
  bar: number;
  startNum: number;
  startDen: number;
  duration: number;
  dots: number;
}

function HelloRenderer() {
  const [showBetweenBarEdit, setShowBetweenBarEdit] = useState(false);
  const [[num, den], setTs] = useState([3, 4]);
  const [hover, setHover] = useState(false);
  const [notes, setNotes] = useState<Array<Note>>([]);

  return (
    <React.Fragment>
      <SheetMusicView
        onEnter={() => setHover(true)}
        onExit={() => setHover(false)}
        onClick={(time, mode) => {
          if (!time) {
            return;
          }

          if (mode === "between-bars") {
            setShowBetweenBarEdit(true);
            return;
          }
          if (mode === "rnc") {
            setNotes([
              ...notes,
              {
                bar: time[0],
                startNum: time[1],
                startDen: time[2],
                duration: -3,
                dots: 0
              }
            ]);
          }
        }}
      >
        <song freezeSpacing={hover ? 1 : undefined} key={`${num}_${den}`}>
          <staff>
            <between clef={true} tsNum={num} tsDen={den} />
            {Array(4)
              .fill(null)
              .map((_, idx) => (
                <React.Fragment key={idx}>
                  <bar numer={num} denom={den}>
                    {notes
                      .filter(note => note.bar === idx)
                      .map(({ dots, duration, startNum, startDen }, idx) => (
                        <rnc
                          key={idx}
                          noteValue={duration}
                          dots={dots}
                          startNum={startNum}
                          startDen={startDen}
                          isNote={true}
                        />
                      ))}
                  </bar>
                  <between
                    barline={idx == 3 ? Barline.Final : Barline.Normal}
                  />
                </React.Fragment>
              ))}
          </staff>
        </song>
      </SheetMusicView>
      {showBetweenBarEdit && (
        <React.Suspense fallback={null}>
          <BetweenBarEdit
            onClose={() => setShowBetweenBarEdit(false)}
            tsNum={num}
            tsDen={den}
            setTs={setTs}
          />
        </React.Suspense>
      )}
    </React.Fragment>
  );
}

export default React.memo(HelloRenderer);
