function v1_to_v2(song: any): any {
  return {
    ...song,
    v: 2,
    global: {
      ...song.global,
      signatures: song.global.between,
      between: undefined,
    },
  };
}

export function update(song: any): any {
  if (song.v === 1) {
    song = v1_to_v2(song);
  }

  return song;
}
