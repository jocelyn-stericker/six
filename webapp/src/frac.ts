import { JsFrac } from "../rust_render_built/index";

export default class Frac {
  private _num: number;
  private _den: number;

  constructor(
    numer: number,
    denom?: number,
    b_numer?: number,
    b_denom?: number,
  ) {
    const frac =
      denom != null && b_numer != null && b_denom != null
        ? JsFrac.add(numer, denom, b_numer, b_denom)
        : JsFrac.reduce(numer, denom || 1);
    this._num = frac[0];
    this._den = frac[1];
  }

  get num(): number {
    return this._num;
  }

  get den(): number {
    return this._den;
  }

  plus(other: Frac): Frac {
    return new Frac(this.num, this.den, other.num, other.den);
  }

  gt(other: Frac): boolean {
    return JsFrac.gt(this.num, this.den, other.num, other.den);
  }

  lt(other: Frac): boolean {
    return JsFrac.lt(this.num, this.den, other.num, other.den);
  }

  eq(other: Frac): boolean {
    return JsFrac.eq(this.num, this.den, other.num, other.den);
  }
}
