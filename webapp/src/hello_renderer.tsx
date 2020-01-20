import React, { useState } from "react";
import SheetMusicView, { NoteValue, Barline } from "./sheet_music_view";

export default function HelloRenderer() {
  const [startNum, setStartNum] = useState(0);

  return (
    <React.Fragment>
      <input
        min={0}
        max={15}
        type="number"
        value={startNum}
        onChange={ev => setStartNum(parseInt(ev.target.value))}
      />
      <SheetMusicView>
        <song>
          <staff>
            <between clef={true} />
            <bar numer={4} denom={4}>
              <rnc
                noteValue={NoteValue.Sixteenth}
                dots={0}
                startNum={startNum}
                startDen={16}
                isNote={true}
              />
            </bar>
            <between barline={Barline.Normal} />
            <bar numer={4} denom={4}>
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
