/// <reference path="./intrinsic_elements.d.ts" /> #

import React from "react";
import { render } from "./renderer";
import { Barline } from "../pkg/index";

render(
  <staff>
    <clef />
    <bar numer={4} denom={4}>
      <rnc noteValue={-3} dots={0} start={[3, 8]} isNote={true} />
    </bar>
    <barline barline={Barline.Normal} />
    <bar numer={4} denom={4}>
      <rnc noteValue={-4} dots={0} start={[1, 4]} isNote={true} />
    </bar>
    <barline barline={Barline.Final} />
  </staff>
);
