import React from "react";
import { Hotkey, Hotkeys, HotkeysTarget } from "@blueprintjs/core";

export interface Props {
  children: JSX.Element;
  onLeft: () => unknown;
  onRight: () => unknown;
  onUp: () => unknown;
  onDown: () => unknown;
  onNote: (note: string, alt: number) => unknown;
  onDuration: (pow2: number) => unknown;
  onBackspace: () => unknown;
}

class EditorHotkeys extends React.Component<Props> {
  render() {
    return this.props.children;
  }

  renderHotkeys() {
    return (
      <Hotkeys>
        <Hotkey
          group="Editor"
          combo="left"
          label="Step back in time"
          onKeyDown={this.props.onLeft}
        />
        <Hotkey
          combo="right"
          group="Editor"
          label="Step forward in time"
          onKeyDown={this.props.onRight}
        />
        <Hotkey
          group="Editor"
          combo="up"
          label="Octave up"
          onKeyDown={ev => {
            ev.preventDefault();
            this.props.onUp();
          }}
        />
        <Hotkey
          group="Editor"
          combo="down"
          label="Octave down"
          onKeyDown={ev => {
            ev.preventDefault();
            this.props.onDown();
          }}
        />
        <Hotkey
          group="Editor"
          combo="a"
          label="Add note: C"
          onKeyDown={() => this.props.onNote("C", 0)}
        />
        <Hotkey
          group="Editor"
          combo="w"
          label="Add note: C#/Db"
          onKeyDown={() => this.props.onNote("C", 1)}
        />
        <Hotkey
          group="Editor"
          combo="s"
          label="Add note: D"
          onKeyDown={() => this.props.onNote("D", 0)}
        />
        <Hotkey
          group="Editor"
          combo="e"
          label="Add note: D#/Eb"
          onKeyDown={() => this.props.onNote("D", 1)}
        />
        <Hotkey
          group="Editor"
          combo="d"
          label="Add note: E"
          onKeyDown={() => this.props.onNote("E", 0)}
        />
        <Hotkey
          group="Editor"
          combo="f"
          label="Add note: F"
          onKeyDown={() => this.props.onNote("F", 0)}
        />
        <Hotkey
          group="Editor"
          combo="t"
          label="Add note: F#/Gb"
          onKeyDown={() => this.props.onNote("F", 1)}
        />
        <Hotkey
          group="Editor"
          combo="g"
          label="Add note: G"
          onKeyDown={() => this.props.onNote("G", 0)}
        />
        <Hotkey
          group="Editor"
          combo="y"
          label="Add note: G#/Ab"
          onKeyDown={() => this.props.onNote("G", 1)}
        />
        <Hotkey
          group="Editor"
          combo="h"
          label="Add note: A"
          onKeyDown={() => this.props.onNote("A", 0)}
        />
        <Hotkey
          group="Editor"
          combo="u"
          label="Add note: A#/Bb"
          onKeyDown={() => this.props.onNote("A", 1)}
        />
        <Hotkey
          group="Editor"
          combo="j"
          label="Add note: B"
          onKeyDown={() => this.props.onNote("B", 0)}
        />
        <Hotkey
          group="Editor"
          combo="1"
          label="Set duration: whole"
          onKeyDown={() => this.props.onDuration(1)}
        />
        <Hotkey
          group="Editor"
          combo="2"
          label="Set duration: half"
          onKeyDown={() => this.props.onDuration(2)}
        />
        <Hotkey
          group="Editor"
          combo="4"
          label="Set duration: quarter"
          onKeyDown={() => this.props.onDuration(4)}
        />
        <Hotkey
          group="Editor"
          combo="5"
          label="Set duration: eighth"
          onKeyDown={() => this.props.onDuration(8)}
        />
        <Hotkey
          group="Editor"
          combo="6"
          label="Set duration: sixteenth"
          onKeyDown={() => this.props.onDuration(16)}
        />
        <Hotkey
          group="Editor"
          combo="backspace"
          label="Remove previous note"
          onKeyDown={this.props.onBackspace}
        />
      </Hotkeys>
    );
  }
}

export default HotkeysTarget(EditorHotkeys);
