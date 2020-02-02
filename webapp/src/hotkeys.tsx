import React from "react";
import { Hotkeys, Hotkey, HotkeysTarget } from "@blueprintjs/core";

export interface Props {
  onUndo: () => void;
  onRedo: () => void;
}

class SixHotkeys extends React.Component<Props> {
  render() {
    return <div />;
  }

  renderHotkeys() {
    return (
      <Hotkeys>
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

export default HotkeysTarget(SixHotkeys);
