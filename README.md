# Six Eight

Six Eight will be a fast, lightweight lead sheet (chords + melody + lyrics) editor
built using Rust + WebAssembly and a custom React reconciler.

**Status**: Six Eight is a work-in-progress experiment. It does not work yet.

Six Eight aims to
 - be responsive. No edits should take more than 20ms. This is
   easier said than done. The UI and feature set will be built around making
   20ms edits possible.
 - apply music typesetting, metre, and "grammar" rules automatically. It might
   be possible to move items around or break these "rules", but customizability
   is a secondary goal.
 - be easy to get started in and learn. There are a number of sheet music
   programs that can make lead sheets, but they are hard to learn. You should
   be able to figure out how to make a lead sheet in Six Eight by moving your
   mouse around. It's possible that talented users could make lead sheets
   faster in other programs. This goal also disqualifies any mandatory
   login/download/onboarding flow.

## No Contributions

I am developing this openly, but Six Eight is not accepting contributions, and
Six Eight is not intended to be used in other projects.

I am also not interested in feedback yet, given that the app does not work yet.

## License

Everything in this repo is licensed under the [GNU Affero General Public
License v3.0 or later](https://github.com/jnetterf/six/blob/master/LICENSE.txt).

## Run the dev server

 - Install Rust, node, and wasm-pack.
 - `cd ./webapp`
 - `npm install`
 - `npm start`.
