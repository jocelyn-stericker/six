import React, { useState } from "react";
import SheetMusicView, { NoteValue, Barline } from "./sheet_music_view";

interface Note {
  startNum: number;
  duration: number;
  dots: number;
}

function HelloRenderer() {
  const [num, _setNum] = useState(4);
  const [notes, _setNotes] = useState<Array<Note>>([
    { startNum: 0, duration: -3, dots: 0 }
  ]);

  return (
    <React.Fragment>
      <SheetMusicView>
        <song>
          <staff>
            <between clef={true} tsNum={num} tsDen={4} />
            <bar numer={4} denom={4}>
              {notes.map(({ dots, duration, startNum }, idx) => (
                <rnc
                  key={idx}
                  noteValue={duration}
                  dots={dots}
                  startNum={startNum}
                  startDen={16}
                  isNote={true}
                />
              ))}
            </bar>
            <between barline={Barline.Normal} />
            <bar numer={num} denom={4}>
              <rnc
                noteValue={NoteValue.Sixteenth}
                dots={0}
                startNum={1}
                startDen={16}
                isNote={true}
              />
            </bar>
            <between barline={Barline.Final} />
          </staff>
        </song>
      </SheetMusicView>
    </React.Fragment>
  );
}

export default React.memo(HelloRenderer);
