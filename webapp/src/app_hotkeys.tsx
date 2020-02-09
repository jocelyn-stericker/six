import React from "react";
import { Hotkeys, Hotkey, HotkeysTarget } from "@blueprintjs/core";

export interface Props {
  onUndo: () => void;
  onRedo: () => void;
  onSetTool: (tool: "notes" | "bars") => void;
}

class AppHotkeys extends React.Component<Props> {
  render() {
    return <div />;
  }

  renderHotkeys() {
    return (
      <Hotkeys>
        <Hotkey
          global={true}
          combo="n"
          label="Mode: Add Notes"
          onKeyDown={() => this.props.onSetTool("notes")}
        />
        <Hotkey
          global={true}
          combo="b"
          label="Mode: Edit Bars and Signatures"
          onKeyDown={() => this.props.onSetTool("bars")}
        />
        <Hotkey
          global={true}
          combo="mod + z"
          label="Undo"
          onKeyDown={this.props.onUndo}
        />
        <Hotkey
          global={true}
          combo="mod + shift + z"
          label="Redo"
          onKeyDown={this.props.onRedo}
        />
      </Hotkeys>
    );
  }
}

export default HotkeysTarget(AppHotkeys);
