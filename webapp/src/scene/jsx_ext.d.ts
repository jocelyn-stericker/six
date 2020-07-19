declare namespace JSX {
  interface IntrinsicElements {
    song: import("./reconciler").SongProps;
    staff: import("./reconciler").StaffProps;
    bar: import("./reconciler").BarProps;
    chord: import("./reconciler").ChordProps;
    signature: import("./reconciler").SignatureProps;
    cursor: import("./reconciler").CursorProps;
  }
}
