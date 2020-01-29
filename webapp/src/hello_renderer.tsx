import React, { useState } from "react";
import SheetMusicView, { Barline } from "./sheet_music_view";

interface Note {
  bar: number;
  startNum: number;
  startDen: number;
  duration: number;
  dots: number;
}

function HelloRenderer() {
  const [num, _setNum] = useState(4);
  const [notes, setNotes] = useState<Array<Note>>([
    { bar: 0, startNum: 0, startDen: 16, duration: -3, dots: 0 },
    { bar: 1, startNum: 1, startDen: 16, duration: -4, dots: 0 }
  ]);

  return (
    <React.Fragment>
      <SheetMusicView
        onClick={time => {
          if (time) {
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
        <song>
          <staff>
            <between clef={true} tsNum={num} tsDen={4} />
            <bar numer={4} denom={4}>
              {notes
                .filter(note => note.bar === 0)
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
            <between barline={Barline.Normal} />
            <bar numer={num} denom={4}>
              {notes
                .filter(note => note.bar === 1)
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
            <between barline={Barline.Final} />
          </staff>
        </song>
      </SheetMusicView>
    </React.Fragment>
  );
}

export default React.memo(HelloRenderer);
