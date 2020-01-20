/// <reference path="./intrinsic_elements.d.ts" /> #

import { Render, Barline } from "../pkg/index";
import { unstable_now as now } from "scheduler";
import ReactReconciler from "react-reconciler";

export { Render, Barline } from "../pkg/index";

export enum NoteValue {
  Maxima = 3,
  Longa = 2,
  DoubleWhole = 1,
  Whole = 0,
  Half = -1,
  Quarter = -2,
  Eighth = -3,
  Sixteenth = -4,
  ThirtySecond = -5,
  SixtyFourth = -6,
  HundredTwentyEighth = -7,
  TwoHundredFiftySixth = -8
}

interface Instance {
  type: "song" | "staff" | "bar" | "between" | "rnc";
  container: Render;
  entity: number;
}

export interface SongProps {}

export interface StaffProps {
  children: any;
}

export interface BarProps {
  numer: number;
  denom: number;
  children: any;
}

export interface BetweenBarsProps {
  clef?: boolean;
  barline?: Barline | undefined;
}

export interface RncProps {
  noteValue: number;
  dots: number;
  startNum: number;
  startDen: number;
  isNote: boolean;
}

// TODO: dedupe with JSX.IntrinsicElements
type CreateInstanceParam =
  | { type: "song"; props: SongProps }
  | { type: "staff"; props: StaffProps }
  | { type: "bar"; props: BarProps }
  | { type: "between"; props: BetweenBarsProps }
  | { type: "rnc"; props: RncProps }
  | { type: never; props: never };

function createInstance<T extends keyof JSX.IntrinsicElements>(
  spec: CreateInstanceParam,
  container: Render
): Instance | null {
  if (spec.type === "song") {
    return {
      container,
      type: "song",
      entity: container.song_create()
    };
  }
  if (spec.type === "staff") {
    return {
      container,
      type: "staff",
      entity: container.staff_create()
    };
  }
  if (spec.type === "bar") {
    return {
      container,
      type: "bar",
      entity: container.bar_create(spec.props.numer, spec.props.denom)
    };
  }
  if (spec.type === "between") {
    return {
      container,
      type: "between",
      entity: container.between_bars_create(
        spec.props.barline,
        spec.props.clef || false
      )
    };
  }
  if (spec.type === "rnc") {
    return {
      container,
      type: "rnc",
      entity: container.rnc_create(
        spec.props.noteValue,
        spec.props.dots,
        spec.props.startNum,
        spec.props.startDen,
        spec.props.isNote
      )
    };
  }

  throw new Error(`Invalid type in sheet music reconciler: <${spec.type} />`);
}

function appendChild(parent: Instance, child: Instance) {
  if (!parent || !child || parent.container !== child.container) {
    return;
  }

  if (parent.type === "bar") {
    parent.container.bar_insert(parent.entity, child.entity);
  } else {
    parent.container.child_append(parent.entity, child.entity);
  }
}

const Reconciler = ReactReconciler({
  supportsMutation: true,
  createInstance(type, props, container: Render) {
    // @ts-ignore
    return createInstance({ type, props }, container);
  },
  createTextInstance(
    _text,
    _rootContainerInstance: Render,
    _hostContext,
    _internalInstanceHandle
  ) {
    return null;
  },

  appendChildToContainer(container, child: Instance) {
    container.root_set(child.entity);
  },
  appendChild(parent: Instance, child: Instance) {
    appendChild(parent, child);
  },
  appendInitialChild(parent: Instance, child: Instance) {
    appendChild(parent, child);
  },

  removeChildFromContainer(container: Render, child: Instance) {
    child.container.root_clear(child.entity);
  },
  removeChild(parent: Instance, child: Instance) {
    if (!parent || !child || parent.container !== child.container) {
      return;
    }

    if (parent.type === "bar") {
      child.container.bar_remove(parent.entity, child.entity);
    } else {
      child.container.child_remove(parent.entity, child.entity);
    }
  },
  insertInContainerBefore(
    _container: Render,
    _child: Instance,
    _before: Instance
  ) {
    throw new Error("The root can only have one child");
  },
  insertBefore(parent: Instance, child: Instance, before: Instance) {
    if (parent.type === "bar") {
      parent.container.bar_insert(parent.entity, child.entity);
    } else {
      parent.container.child_insert_before(
        parent.entity,
        before.entity,
        child.entity
      );
    }
  },

  prepareUpdate(
    instance: Instance,
    type,
    // TODO
    oldProps: any,
    newProps: any,
    _rootContainerInstance: Render,
    _currentHostContext
  ) {
    // TODO: ALL the changes
    let changes = [];
    if (
      type === "rnc" &&
      (oldProps.startNum !== newProps.startNum ||
        oldProps.startDen !== newProps.startDen)
    ) {
      changes.push("time");
    }

    return changes;
  },
  commitUpdate(
    instance: Instance,
    // TODO
    updatePayload: any,
    type,
    _oldProps,
    // TODO
    newProps: any,
    _finishedWork
  ) {
    if (type === "rnc") {
      for (const change of updatePayload) {
        if (change === "time") {
          instance.container.rnc_update_time(
            instance.entity,
            newProps.startNum,
            newProps.startDen
          );
        }
      }
    }
  },

  finalizeInitialChildren() {
    return false;
  },
  getChildHostContext() {},
  getPublicInstance() {},
  getRootHostContext() {},
  prepareForCommit() {},
  resetAfterCommit() {},
  shouldSetTextContent() {
    return false;
  },

  now,
  setTimeout,
  clearTimeout,
  shouldDeprioritizeSubtree() {
    return false;
  },
  noTimeout: -1,
  supportsHydration: false,
  supportsPersistence: false,
  isPrimaryRenderer: false,
  cancelDeferredCallback() {},
  scheduleDeferredCallback() {
    return false;
  }
});

const roots = new Map<Render, ReactReconciler.FiberRoot>();

export function render(whatToRender: any, container: Render) {
  let root = roots.get(container);
  if (!root) {
    root = Reconciler.createContainer(container, false, false);
    roots.set(container, root);
  }

  Reconciler.updateContainer(whatToRender, root, null, () => null);
}

Reconciler.injectIntoDevTools({
  bundleType: process.env.NODE_ENV === "production" ? 0 : 1,
  version: "0.10.0",
  rendererPackageName: "six-eight",
  // @ts-ignore
  findFiberByHostInstance() {
    return null;
  }
});
