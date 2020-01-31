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
  const [hover, setHover] = useState(false);
  const [notes, setNotes] = useState<Array<Note>>([
    { bar: 0, startNum: 0, startDen: 16, duration: -3, dots: 0 },
    { bar: 1, startNum: 1, startDen: 16, duration: -4, dots: 0 }
  ]);

  return (
    <React.Fragment>
      <SheetMusicView
        onEnter={() => setHover(true)}
        onExit={() => setHover(false)}
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
        <song freezeSpacing={hover ? 1 : undefined}>
          <staff>
            <between clef={true} tsNum={num} tsDen={4} />
            {Array(4)
              .fill(null)
              .map((_, idx) => (
                <React.Fragment key={idx}>
                  <bar numer={4} denom={4}>
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
    </React.Fragment>
  );
}

export default React.memo(HelloRenderer);
