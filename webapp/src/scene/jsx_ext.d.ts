declare namespace JSX {
  interface IntrinsicElements {
    song: import("./reconciler").SongProps;
    staff: import("./reconciler").StaffProps;
    bar: import("./reconciler").BarProps;
    rnc: import("./reconciler").RncProps;
    between: import("./reconciler").BetweenBarsProps;
    cursor: import("./reconciler").CursorProps;
  }
}
