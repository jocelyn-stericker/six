import React from "react";
import { Hotkeys as BPHotkeys, Hotkey, HotkeysTarget } from "@blueprintjs/core";

export interface Props {
  onUndo: () => void;
  onRedo: () => void;
}

class Hotkeys extends React.Component<Props> {
  render() {
    return <div />;
  }

  renderHotkeys() {
    return (
      <BPHotkeys>
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
      </BPHotkeys>
    );
  }
}

export default HotkeysTarget(Hotkeys);
