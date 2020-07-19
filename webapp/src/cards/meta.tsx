import React from "react";
import {
  Divider,
  FormGroup,
  HTMLSelect,
  InputGroup,
  NumericInput,
  Radio,
  RadioGroup,
} from "@blueprintjs/core";
import "@blueprintjs/core/lib/css/blueprint.css";
import cx from "classnames";

import {
  Action,
  clearPickupSkip,
  Clef,
  setAuthor,
  setBarCount,
  setClef,
  setKs,
  setPickupSkip,
  setTitle,
  setTs,
  State,
} from "../store";

import "../blueprint/theme.scss";
import css from "./meta.module.scss";

interface Props {
  appState: State;
  dispatch: (action: Action) => void;
}

export default function Meta({ appState, dispatch }: Props) {
  const minBars = appState.song.part.bars.reduce(
    (memo, bar, idx) => (bar.notes.length > 0 ? idx + 1 : memo),
    1,
  );
  const maxBars = 200;
  return (
    <React.Fragment>
      <div className={cx(css.meta, "bp3-dark")}>
        <h2>About this Song</h2>
        <FormGroup
          label="Title"
          labelFor="meta-title"
          className={css.metaGroup}
        >
          <InputGroup
            id="meta-title"
            placeholder="Untitled"
            large={true}
            autoFocus
            value={appState.song.global.title}
            onChange={(ev: React.FormEvent<HTMLInputElement>) => {
              dispatch(setTitle(appState, ev.currentTarget.value));
            }}
          />
        </FormGroup>
        <FormGroup
          label="Author"
          labelFor="meta-author"
          className={css.metaGroup}
        >
          <InputGroup
            id="meta-author"
            placeholder="Anonymous"
            large={true}
            value={appState.song.global.author}
            onChange={(ev: React.FormEvent<HTMLInputElement>) => {
              dispatch(setAuthor(appState, ev.currentTarget.value));
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
              dispatch(setBarCount(appState, count));
            }}
            min={minBars}
            max={maxBars}
            large={true}
            clampValueOnBlur={true}
          />
        </FormGroup>
      </div>
      <div className={cx(css.meta, "bp3-dark")}>
        <h2>First Bar</h2>
        <FormGroup label="Clef">
          <RadioGroup
            inline={true}
            onChange={ev => {
              dispatch(
                setClef(appState, {
                  clef: (ev.currentTarget.value as any) as Clef,
                  beforeBar: 0,
                }),
              );
            }}
            selectedValue={appState.song.global.signatures[0].clef}
          >
            <Radio large={true} label="Treble" value="g" />
            <Radio large={true} label="Bass" value="f" />
            <Radio large={true} label="Percussion" value="percussion" />
          </RadioGroup>
        </FormGroup>
        <FormGroup
          label="Key Signature"
          labelFor="meta-ks"
          className={css.metaGroup}
        >
          <HTMLSelect
            disabled={appState.song.global.signatures[0].clef === "percussion"}
            value={
              appState.song.global.signatures[0].clef === "percussion"
                ? 0
                : appState.song.global.signatures[0].ks
            }
            onChange={ev => {
              dispatch(
                setKs(appState, {
                  ks: parseInt(ev.currentTarget.value),
                  beforeBar: 0,
                }),
              );
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
              dispatch(setTs(appState, { beforeBar: 0, ts: [num, den] }));
            }}
            selectedValue={`${appState.song.global.signatures[0].ts[0]}/${appState.song.global.signatures[0].ts[1]}`}
            className={css.metaTsSelect}
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
        <div style={{ height: 8 }} />
        <FormGroup
          label={`Does the first bar have ${appState.song.global.signatures[0].ts[0]} beats?`}
          labelFor="meta-pickup"
        >
          <HTMLSelect
            value={appState.song.global.pickupSkip == null ? "full" : "pickup"}
            onChange={ev => {
              if (ev.currentTarget.value === "full") {
                dispatch(clearPickupSkip(appState));
              } else {
                dispatch(setPickupSkip(appState, [1, 4]));
              }
            }}
            options={[
              { label: "Yes", value: "full" },
              { label: "No", value: "pickup" },
            ]}
          />
        </FormGroup>
        {appState.song.global.pickupSkip != null && (
          <>
            <div style={{ height: 8 }} />
            <FormGroup
              label="How many beats does it have?"
              labelFor="meta-pickup-beats"
            >
              <HTMLSelect
                onChange={ev => {
                  const [num, den] = ev.currentTarget.value
                    .split("/")
                    .map(ts => parseInt(ts));
                  dispatch(setPickupSkip(appState, [num, den]));
                }}
                value={`${appState.song.global.pickupSkip[0]}/${appState.song.global.pickupSkip[1]}`}
                options={[
                  {
                    label: "\u00bd",
                    value: `${appState.song.global.signatures[0].ts[0] * 2 -
                      1}/${appState.song.global.signatures[0].ts[1] * 2}`,
                  },
                  ...Array(appState.song.global.signatures[0].ts[0] - 1)
                    .fill(null)
                    .map((_, i) => [
                      {
                        label: String(i + 1),
                        value: `${appState.song.global.signatures[0].ts[1] -
                          i -
                          1}/${appState.song.global.signatures[0].ts[1]}`,
                      },
                      {
                        label: `${i + 1} \u00bd`,
                        value: `${appState.song.global.signatures[0].ts[1] * 2 -
                          (i * 2 + 1) -
                          2}/${appState.song.global.signatures[0].ts[1] * 2}`,
                      },
                    ])
                    .reduce((memo, item) => [...memo, ...item], []),
                ]}
              />
            </FormGroup>
          </>
        )}
      </div>
    </React.Fragment>
  );
}
