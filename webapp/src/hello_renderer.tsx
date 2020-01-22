import React, { useState } from "react";
import SheetMusicView, { NoteValue, Barline } from "./sheet_music_view";

interface Note {
  startNum: number;
  duration: number;
  dots: number;
}

export default function HelloRenderer() {
  const [num, setNum] = useState(4);
  const [notes, setNotes] = useState<Array<Note>>([
    { startNum: 0, duration: -3, dots: 0 }
  ]);

  return (
    <React.Fragment>
      <div>
        <input
          min={1}
          max={15}
          type="number"
          value={num}
          onChange={ev => {
            setNum(parseInt(ev.target.value));
          }}
        />
      </div>
      {notes.map((note, idx) => (
        <div key={idx}>
          <input
            min={0}
            max={15}
            type="number"
            value={note.startNum}
            onChange={ev => {
              let newNotes = [...notes];
              newNotes[idx] = {
                startNum: parseInt(ev.target.value),
                duration: note.duration,
                dots: note.dots
              };
              setNotes(newNotes);
            }}
          />
          <input
            min={-6}
            max={0}
            type="number"
            value={note.duration}
            onChange={ev => {
              let newNotes = [...notes];
              newNotes[idx] = {
                duration: parseInt(ev.target.value),
                startNum: note.startNum,
                dots: note.dots
              };
              setNotes(newNotes);
            }}
          />
          <input
            min={0}
            max={2}
            type="number"
            value={note.dots}
            onChange={ev => {
              let newNotes = [...notes];
              newNotes[idx] = {
                dots: parseInt(ev.target.value),
                startNum: note.startNum,
                duration: note.duration
              };
              setNotes(newNotes);
            }}
          />
        </div>
      ))}
      <button
        onClick={() =>
          setNotes([
            ...notes,
            { startNum: notes.length, duration: -3, dots: 0 }
          ])
        }
      >
        Add
      </button>
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
