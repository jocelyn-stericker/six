import React from "react";
import {
  RadioGroup,
  Radio,
  FormGroup,
  InputGroup,
  Divider,
  NumericInput,
  HTMLSelect,
} from "@blueprintjs/core";
import { Action, State } from "./store";

import "@blueprintjs/core/lib/css/blueprint.css";
import "./blueprint/theme.css";
import "./meta.css";

interface Props {
  appState: State;
  dispatch: (action: Action) => void;
}

export default function Meta({ appState, dispatch }: Props) {
  const minBars = appState.song.part.bars.reduce(
    (memo, bar, idx) => (bar.notes.length > 0 ? idx + 1 : memo),
    1,
  );
  const maxBars = 20;
  return (
    <React.Fragment>
      <div className="meta bp3-dark">
        <h2>About this Song</h2>
        <FormGroup label="Title" labelFor="meta-title" className="meta-group">
          <InputGroup
            id="meta-title"
            placeholder="Untitled"
            large={true}
            autoFocus
            value={appState.song.global.title}
            onChange={(ev: React.FormEvent<HTMLInputElement>) => {
              dispatch({
                type: "SET_TITLE",
                title: ev.currentTarget.value,
                prevTitle: appState.song.global.title,
              });
            }}
          />
        </FormGroup>
        <FormGroup label="Author" labelFor="meta-author" className="meta-group">
          <InputGroup
            id="meta-author"
            placeholder="Anonymous"
            large={true}
            value={appState.song.global.author}
            onChange={(ev: React.FormEvent<HTMLInputElement>) => {
              dispatch({
                type: "SET_AUTHOR",
                author: ev.currentTarget.value,
                prevAuthor: appState.song.global.author,
              });
            }}
          />
        </FormGroup>
        <FormGroup label="Number of Bars">
          <NumericInput
            value={appState.song.part.bars.length}
            onValueChange={count => {
              if (count < minBars || count > maxBars) {
                return;
              }
              dispatch({
                type: "SET_BAR_COUNT",
                prevCount: appState.song.part.bars.length,
                count,
              });
            }}
            min={minBars}
            max={maxBars}
            large={true}
            clampValueOnBlur={true}
          />
        </FormGroup>
      </div>
      <div className="meta bp3-dark">
        <h2>First Bar</h2>
        <FormGroup label="Clef">
          <RadioGroup
            inline={true}
            onChange={ev => {
              dispatch({
                type: "SET_CLEF",
                prevClef: appState.song.global.clef as any,
                clef: ev.currentTarget.value as any,
              });
            }}
            selectedValue={appState.song.global.clef}
          >
            <Radio large={true} label="Treble" value="g" />
            <Radio large={true} label="Bass" value="f" />
            <Radio large={true} label="Percussion" value="percussion" />
          </RadioGroup>
        </FormGroup>
        <FormGroup
          label="Key Signature"
          labelFor="meta-ks"
          className="meta-group"
        >
          <HTMLSelect
            disabled={appState.song.global.clef === "percussion"}
            value={
              appState.song.global.clef === "percussion"
                ? 0
                : appState.song.global.ks
            }
            onChange={ev => {
              dispatch({
                type: "SET_KS",
                prevKs: appState.song.global.ks as any,
                ks: parseInt(ev.currentTarget.value),
              });
            }}
            options={[
              { label: "G♭ Major / e♭ minor (6♭)", value: -6 },
              { label: "D♭ Major / b♭ minor (5♭)", value: -5 },
              { label: "A♭ Major / f minor (4♭)", value: -4 },
              { label: "E♭ Major / c minor (3♭)", value: -3 },
              { label: "B♭ Major / g minor (2♭)", value: -2 },
              { label: "F Major / d minor (♭)", value: -1 },
              { label: "C Major / a minor", value: 0 },
              { label: "G Major / e minor (♯)", value: 1 },
              { label: "D Major / b minor (2♯)", value: 2 },
              { label: "A Major / f♯ minor (3♯)", value: 3 },
              { label: "E Major / c♯ minor (4♯)", value: 4 },
              { label: "B Major / g♯ minor (5♯)", value: 5 },
              { label: "F♯ Major / d♯ minor (6♯)", value: 6 },
            ]}
          />
        </FormGroup>
        <div style={{ height: 8 }} />
        <FormGroup label="Time Signature">
          <RadioGroup
            onChange={ev => {
              const [num, den] = ev.currentTarget.value
                .split("/")
                .map(ts => parseInt(ts));
              dispatch({
                type: "SET_TS",
                num,
                den,
                prevNum: appState.song.global.tsNum,
                prevDen: appState.song.global.tsDen,
              });
            }}
            selectedValue={`${appState.song.global.tsNum}/${appState.song.global.tsDen}`}
            className="meta-ts-select"
          >
            <Radio label="4/4" value="4/4" />
            <Radio label="2/2" value="2/2" />
            <Radio label="2/4" value="2/4" />
            <Radio label="4/8" value="4/8" />
            <Divider />
            <Radio label="3/4" value="3/4" />
            <Radio label="3/8" value="3/8" />
            <Divider />
            <Radio label="6/8" value="6/8" />
            <Radio label="6/4" value="6/4" />
            <Radio label="6/16" value="6/16" />
            <Divider />
            <Radio label="9/8" value="9/8" />
            <Radio label="12/8" value="12/8" />
          </RadioGroup>
        </FormGroup>
      </div>
    </React.Fragment>
  );
}
