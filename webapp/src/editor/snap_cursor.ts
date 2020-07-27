import { NativeSixDom } from "../scene";

export default function snapCursor(
  api: NativeSixDom,
  staff: number,
  barIdx: number,
  t: [number, number],
) {
  return api.staff_time_cursor_add(staff, barIdx, t[0], t[1], 0, 1);
}
