import React, { forwardRef, useImperativeHandle, useState } from "react";
import cx from "classnames";
import { Icon } from "@blueprintjs/core";

import css from "./keyboard.module.scss";
import appCss from "../app.module.scss";

export interface KeyboardRef {
  onPhysicalKeyPress(key: string, alt?: number): void;
}

function Whole() {
  return (
    <svg viewBox="0 0 9 9" height="9mm" width="9mm">
      <g transform="translate(0,-288)">
        <path d="m 4.547625,291.34114 c -0.557257,0.004 -1.1492418,0.0967 -1.6050579,0.44022 -0.2610986,0.19168 -0.4353266,0.51961 -0.3747731,0.84754 0.045921,0.39576 0.3649445,0.69971 0.7223673,0.83886 0.5504001,0.21823 1.1599313,0.21466 1.740715,0.16088 0.4337156,-0.0529 0.9055312,-0.16415 1.2006076,-0.51275 0.189232,-0.23665 0.2686635,-0.57726 0.1643176,-0.86622 -0.1615777,-0.36778 -0.5336679,-0.5818 -0.8913432,-0.72463 -0.3047206,-0.11639 -0.6302164,-0.18274 -0.9568333,-0.1839 z m -0.9630833,0.58208 c 0.067528,-0.25626 0.3502844,-0.35697 0.5893332,-0.36573 0.35966,-0.0475 0.715331,0.12989 0.9412813,0.4041 0.2439067,0.29084 0.41105,0.70093 0.2914215,1.07884 -0.063703,0.21078 -0.2584424,0.35617 -0.4719996,0.38834 -0.4116086,0.0879 -0.8218121,-0.14994 -1.0683489,-0.46763 -0.2190975,-0.28036 -0.3706859,-0.65169 -0.2921675,-1.01014 z" />
      </g>
    </svg>
  );
}

function Half() {
  return (
    <svg width="9mm" height="9mm" viewBox="0 0 9 9">
      <g transform="translate(0,-288)">
        <path d="m 4.0078751,296.818 c 0.6873953,0.0237 1.4092703,-0.30552 1.7297919,-0.93528 0.1887615,-0.31458 0.2229312,-0.68264 0.206958,-1.0406 0,-2.22004 0,-4.44008 0,-6.66012 -0.092473,0.0225 -0.2953578,-0.0653 -0.254,0.0911 0,2.10748 0,4.21496 0,6.32244 -0.4144043,-0.33818 -1.0101977,-0.25609 -1.4844659,-0.10739 -0.6753591,0.23078 -1.2034515,0.92961 -1.1401364,1.6549 0.056046,0.45024 0.525375,0.69016 0.9418524,0.67499 z m 0.22225,-1.67217 c 0.3454621,-0.1747 0.7122522,-0.42555 1.1163762,-0.36578 0.3738766,0.11298 0.3459106,0.60139 0.041183,0.78591 -0.4271187,0.38774 -0.934737,0.71245 -1.4957993,0.86142 -0.3121571,0.0831 -0.6804989,-0.2644 -0.5071034,-0.57461 0.1874334,-0.32432 0.5364213,-0.51656 0.8453438,-0.70694 z" />
      </g>
    </svg>
  );
}

function Quarter() {
  return (
    <svg viewBox="0 0 9 9" height="9mm" width="9mm">
      <g transform="translate(0,-288)">
        <path d="M 5.6535834,294.63253 C 5.48425,294.48437 5.2514167,294.41028 4.97625,294.41028 c -1.0477499,0 -1.8838332,0.77259 -1.8838332,1.59809 0,0.49741 0.4021666,0.79375 0.9207499,0.79375 0.8466667,0 1.8944166,-0.79375 1.8944166,-1.59809 v -7.00616 H 5.6535834 Z" />
      </g>
    </svg>
  );
}

function Eighth() {
  return (
    <svg width="9mm" height="9mm" viewBox="0 0 9 9">
      <g transform="translate(0,-288)">
        <path d="m 5.9234582,290.58443 c -0.4475053,-0.67457 -0.8075308,-1.42209 -0.9342685,-2.2274 -0.1103392,-0.24774 -0.4107728,-0.12583 -0.3251481,0.11196 0,2.04923 0,4.09846 0,6.14769 -0.4218281,-0.34057 -1.0297863,-0.24089 -1.4972513,-0.0552 -0.6183303,0.25623 -1.1594473,0.89451 -1.0456218,1.59733 0.096826,0.48683 0.6619133,0.68344 1.1039804,0.60285 0.6834343,-0.11121 1.3457529,-0.55225 1.6209171,-1.20095 0.122278,-0.3727 0.054213,-0.77092 0.071976,-1.15686 0,-1.32254 0,-2.64509 0,-3.96763 0.6063847,0.44005 1.0167561,1.09764 1.3538708,1.75466 0.2778912,0.61164 0.2926346,1.31429 0.2098166,1.97067 0.00612,0.29652 -0.4041204,0.83747 0.030923,0.95598 0.2970929,-0.18375 0.2249542,-0.6317 0.3237419,-0.93521 0.239346,-1.27064 -0.2418153,-2.53802 -0.9129359,-3.59793 z" />
      </g>
    </svg>
  );
}

function Sixteenth() {
  return (
    <svg width="9mm" height="9mm" viewBox="0 0 9 9">
      <g transform="translate(0,-288)">
        <path d="m 6.7807082,292.83868 c 0.015243,-0.30681 0.1821199,-0.60211 0.1939468,-0.91885 0.076336,-0.57158 -0.097778,-1.17409 -0.5002023,-1.59348 -0.3096485,-0.38411 -0.6737056,-0.71537 -1.0419364,-1.04102 -0.272448,-0.25905 -0.4118132,-0.62226 -0.4697023,-0.98705 -0.098025,-0.11241 -0.378955,-0.12883 -0.2987724,0.0839 0,2.07815 0,4.15631 0,6.23447 -0.4218281,-0.34057 -1.0297863,-0.24089 -1.4972513,-0.0552 -0.6183303,0.25623 -1.1594473,0.89451 -1.0456218,1.59733 0.096826,0.48683 0.6619133,0.68344 1.1039804,0.60285 0.6834606,-0.11121 1.3457058,-0.55224 1.6209171,-1.20095 0.1218239,-0.36783 0.054472,-0.7611 0.071976,-1.14214 0,-0.9394 0,-1.87879 0,-2.81818 0.5617341,0.003 1.0330302,0.39201 1.3013779,0.86122 0.4356685,0.64845 0.5481838,1.46415 0.4145121,2.22382 -0.1482985,0.2558 0.2036996,0.66682 0.3385375,0.25536 0.053892,-0.51005 0.090335,-1.03409 5.064e-4,-1.54188 -0.042431,-0.19331 -0.108667,-0.38103 -0.1922673,-0.56027 z m -0.127,-0.9525 c 0.017846,0.36964 -0.297147,0.27342 -0.3820542,0.009 -0.2941292,-0.41942 -0.7140723,-0.72735 -1.0165403,-1.13806 -0.092926,-0.17632 -0.4446672,-0.66489 -0.072489,-0.68035 0.5855388,0.0248 1.0136159,0.51708 1.3115065,0.97381 0.1422533,0.25128 0.1956518,0.55032 0.1595768,0.83594 z" />
      </g>
    </svg>
  );
}

function Dot() {
  return (
    <svg width="9mm" height="9mm" viewBox="0 0 9 9">
      <g transform="translate(0,-288)">
        <g transform="matrix(0.50127243,0,0,0.50127243,2.2442741,145.87781)">
          <path d="m 6.9115627,292.5 c 0,-1.32636 -1.0852033,-2.41156 -2.4115628,-2.41156 -1.3263596,0 -2.4115628,1.0852 -2.4115628,2.41156 0,1.32636 1.0852032,2.41157 2.4115628,2.41157 1.3263595,0 2.4115628,-1.08521 2.4115628,-2.41157 z" />
        </g>
      </g>
    </svg>
  );
}

function Undo() {
  return (
    <svg width="9mm" height="9mm" viewBox="-4 -4 24 24">
      <desc>undo</desc>
      <path
        d="M4 11c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm7-7H3.41L4.7 2.71c.19-.18.3-.43.3-.71a1.003 1.003 0 00-1.71-.71l-3 3C.11 4.47 0 4.72 0 5c0 .28.11.53.29.71l3 3a1.003 1.003 0 001.42-1.42L3.41 6H11c1.66 0 3 1.34 3 3s-1.34 3-3 3H7v2h4c2.76 0 5-2.24 5-5s-2.24-5-5-5z"
        fillRule="evenodd"
      ></path>
    </svg>
  );
}

function RadioButton(props: {
  selected: boolean;
  title: string;
  icon: React.ReactChild;
  shortcut: React.ReactChild;
  onSelect: () => unknown;
}) {
  return (
    <label
      className={cx(
        css.radioButton,
        props.selected && css.selected,
        css.button,
      )}
      title={props.title}
    >
      <input
        type="radio"
        className={css.radio}
        value={props.selected ? "on" : ""}
        onChange={props.onSelect}
      />
      <div className={css.buttonContent}>
        {props.icon}
        <div className={cx(css.radioButtonShortcut, css.mnemonic)}>
          {props.shortcut}
        </div>
      </div>
    </label>
  );
}

function ActionButton(props: {
  title: string;
  icon: React.ReactChild;
  shortcut?: React.ReactChild;
  bigger?: boolean;
  flex?: boolean;
  onClick: () => unknown;
  active?: boolean;
}) {
  return (
    <button
      onClick={props.onClick}
      className={cx(
        css.actionButton,
        css.button,
        props.bigger && css.biggerKey,
        props.flex && css.flexKey,
        props.active && css.triggered,
      )}
      title={props.title}
    >
      <div
        className={cx(
          !props.flex && css.actionButton,
          !props.flex && css.buttonContent,
          props.flex && css.textKeyContent,
        )}
      >
        {props.icon}
        {props.shortcut && (
          <div className={cx(css.radioButtonShortcut, css.mnemonic)}>
            {props.shortcut}
          </div>
        )}
      </div>
    </button>
  );
}

export interface Props {
  onLeft: () => unknown;
  onRight: () => unknown;
  onUp: () => unknown;
  onDown: () => unknown;
  onDuration: (pow2: number) => unknown;
  duration: number;
  onUndo: () => unknown;
  onNote: (base: string, alt: number) => unknown;
}

const Keyboard = forwardRef(function Keyboard(
  props: Props,
  ref: React.Ref<KeyboardRef>,
) {
  const [activeKeys, setActiveKeys] = useState<{ [key: string]: number }>({});

  useImperativeHandle(ref, () => ({
    onPhysicalKeyPress(key: string, alt: number) {
      const name = key + (alt || "");
      setActiveKeys(keys => ({ ...keys, [name]: (keys[name] ?? 0) + 1 }));
      setTimeout(() => {
        setActiveKeys(keys => ({ ...keys, [name]: (keys[name] ?? 0) - 1 }));
      }, 120);
    },
  }));
  return (
    <div
      className={cx(css.keyboard, appCss.editorKeyboard)}
      onTouchStart={() => {}}
    >
      <div className={css.smallRow}>
        <RadioButton
          title="Whole &mdash; 1"
          selected={props.duration === 1}
          icon={<Whole />}
          shortcut="1"
          onSelect={() => props.onDuration(1)}
        />
        <div className={css.buttonGap} />
        <RadioButton
          title="Half &mdash; 2"
          selected={props.duration === 2}
          icon={<Half />}
          shortcut="2"
          onSelect={() => props.onDuration(2)}
        />
        <div className={css.buttonGap} />
        <RadioButton
          title="Quarter &mdash; 4"
          selected={props.duration === 4}
          icon={<Quarter />}
          shortcut="4"
          onSelect={() => props.onDuration(4)}
        />
        <div className={css.buttonGap} />
        <RadioButton
          title="Eighth &mdash; 5"
          selected={props.duration === 8}
          icon={<Eighth />}
          shortcut="5"
          onSelect={() => props.onDuration(8)}
        />
        <div className={css.buttonGap} />
        <RadioButton
          title="Sixteenth &mdash; 6"
          selected={props.duration === 16}
          icon={<Sixteenth />}
          shortcut="6"
          onSelect={() => props.onDuration(16)}
        />
        <div className={css.buttonGap} />
        <RadioButton
          title="Dot &mdash; "
          selected={false}
          icon={<Dot />}
          shortcut="."
          onSelect={() => alert("TODO")}
        />
        <div className={css.buttonGap} />
        <ActionButton
          onClick={props.onUndo}
          title="Undo &mdash; ctrl/cmd+z"
          icon={<Undo />}
          shortcut="c+z"
          bigger={true}
        />
      </div>
      <div className={css.pianokeys}>
        <div className={css.blackKeys}>
          <div className={css.flex10} />
          <button
            className={cx(
              css.blackKey,
              css.flex12,
              activeKeys["C1"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("C", 1)}
          >
            <div className={css.blackKeyShortcut}>w</div>
          </button>
          <div className={css.flex11} />
          <button
            className={cx(
              css.blackKey,
              css.flex12,
              activeKeys["D1"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("D", 1)}
          >
            <div className={css.blackKeyShortcut}>e</div>
          </button>
          <div className={css.flex22} />
          <button
            className={cx(
              css.blackKey,
              css.flex12,
              activeKeys["F1"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("F", 1)}
          >
            <div className={css.blackKeyShortcut}>t</div>
          </button>
          <div className={css.flex9} />
          <button
            className={cx(
              css.blackKey,
              css.flex12,
              activeKeys["G1"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("G", 1)}
          >
            <div className={css.blackKeyShortcut}>y</div>
          </button>
          <div className={css.flex9} />
          <button
            className={cx(
              css.blackKey,
              css.flex12,
              activeKeys["A1"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("A", 1)}
          >
            <div className={css.blackKeyShortcut}>u</div>
          </button>
          <div className={css.flex10} />
        </div>
        <div className={css.whiteKeys}>
          <button
            className={cx(
              css.whiteKey,
              css.flex17,
              activeKeys["C"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("C", 0)}
          >
            <div className={cx(css.whiteKeyShortcut)}>A</div>
          </button>
          <div className={css.flex2} />
          <button
            className={cx(
              css.whiteKey,
              css.flex17,
              activeKeys["D"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("D", 0)}
          >
            <div className={css.whiteKeyShortcut}>S</div>
          </button>
          <div className={css.flex2} />
          <button
            className={cx(
              css.whiteKey,
              css.flex17,
              activeKeys["E"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("E", 0)}
          >
            <div className={css.whiteKeyShortcut}>D</div>
          </button>
          <div className={css.flex2} />
          <button
            className={cx(
              css.whiteKey,
              css.flex17,
              activeKeys["F"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("F", 0)}
          >
            <div className={cx(css.whiteKeyShortcut, css.mnemonic)}>F</div>
          </button>
          <div className={css.flex2} />
          <button
            className={cx(
              css.whiteKey,
              css.flex17,
              activeKeys["G"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("G", 0)}
          >
            <div className={cx(css.whiteKeyShortcut, css.mnemonic)}>G</div>
          </button>
          <div className={css.flex2} />
          <button
            className={cx(
              css.whiteKey,
              css.flex17,
              activeKeys["A"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("A", 0)}
          >
            <div className={css.whiteKeyShortcut}>H</div>
          </button>
          <div className={css.flex2} />
          <button
            className={cx(
              css.whiteKey,
              css.flex17,
              activeKeys["B"] > 0 && css.triggered,
            )}
            onClick={() => props.onNote("B", 0)}
          >
            <div className={css.whiteKeyShortcut}>J</div>
          </button>
        </div>
      </div>
      <div className={css.smallRow}>
        <ActionButton
          onClick={props.onDown}
          title="Octave down &mdash; down arrow"
          icon={
            <>
              8<sup>vb</sup>
            </>
          }
          shortcut={<Icon icon="arrow-down" iconSize={8} />}
          flex={true}
          active={activeKeys["down"] > 0}
        />
        <div className={css.buttonGap} />
        <ActionButton
          onClick={props.onLeft}
          title="Move to earlier note or rest"
          icon={<Icon icon="arrow-left" iconSize={16} />}
          bigger={true}
          flex={true}
          active={activeKeys["back"] > 0}
        />
        <div className={css.buttonGap} />
        <ActionButton
          onClick={props.onRight}
          title="Move to later note or rest"
          icon={<Icon icon="arrow-right" iconSize={16} />}
          bigger={true}
          flex={true}
          active={activeKeys["forward"] > 0}
        />
        <div className={css.buttonGap} />
        <ActionButton
          onClick={props.onUp}
          title="Octave up &mdash; up arrow"
          icon={
            <>
              8<sup>va</sup>
            </>
          }
          shortcut={<Icon icon="arrow-up" iconSize={8} />}
          flex={true}
          active={activeKeys["up"] > 0}
        />
        <div className={css.buttonGap} />
        <ActionButton
          onClick={() => alert("TODO")}
          title="Add chord"
          icon={<>Cm</>}
          shortcut="shift"
          flex={true}
        />
        <div className={css.buttonGap} />
        <ActionButton
          onClick={() => alert("TODO")}
          title="Add lyrics"
          icon={<>lyr</>}
          shortcut="l"
          flex={true}
        />
        <div className={css.buttonGap} />
        <ActionButton
          onClick={() => alert("TODO")}
          title="Backspace"
          icon={<span className={css.backspace}>backspace</span>}
          flex={true}
          bigger={true}
        />
      </div>
    </div>
  );
});

export default Keyboard;
