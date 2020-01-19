import { Render, Barline } from "../pkg/index";
import { unstable_now as now } from "scheduler";
import ReactReconciler from "react-reconciler";

interface Instance {
  type: "staff" | "bar" | "barline" | "clef" | "rnc";
  container: Render;
  entity: number;
}

export interface StaffProps {
  children: any;
}

export interface BarProps {
  numer: number;
  denom: number;
  children: any;
}

export interface BarlineProps {
  barline: Barline;
}

export interface ClefProps {}

export interface RncProps {
  noteValue: number;
  dots: number;
  start: [number, number];
  isNote: boolean;
}

// TODO: dedupe with JSX.IntrinsicElements
type CreateInstanceParam =
  | { type: "staff"; props: StaffProps }
  | { type: "bar"; props: BarProps }
  | { type: "barline"; props: BarlineProps }
  | { type: "clef"; props: ClefProps }
  | { type: "rnc"; props: RncProps };

function createInstance<T extends keyof JSX.IntrinsicElements>(
  spec: CreateInstanceParam,
  container: Render
): Instance | null {
  if (spec.type === "staff") {
    return {
      container: container,
      type: "staff",
      entity: container.append_staff()
    };
  }
  if (spec.type === "bar") {
    return {
      container: container,
      type: "bar",
      entity: container.create_bar(spec.props.numer, spec.props.denom)
    };
  }
  if (spec.type === "barline") {
    return {
      container: container,
      type: "barline",
      entity: container.create_barline(spec.props.barline)
    };
  }
  if (spec.type === "clef") {
    return {
      container: container,
      type: "clef",
      entity: container.create_clef()
    };
  }
  if (spec.type === "rnc") {
    return {
      container: container,
      type: "rnc",
      entity: container.create_rnc(
        spec.props.noteValue,
        spec.props.dots,
        spec.props.start[0],
        spec.props.start[1],
        spec.props.isNote
      )
    };
  }

  return null;
}

function appendChild(parent: Instance, child: Instance) {
  if (!parent || !child || parent.container !== child.container) {
    return;
  }

  if (parent.type === "staff") {
    parent.container.append_to_staff(parent.entity, child.entity);
  }
  if (parent.type === "bar" && child.type === "rnc") {
    parent.container.append_rnc(parent.entity, child.entity);
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

  appendChildToContainer(_container, _child) {
    // TODO
  },
  appendChild(parent: Instance, child: Instance) {
    appendChild(parent, child);
  },
  appendInitialChild(parent: Instance, child: Instance) {
    appendChild(parent, child);
  },

  removeChildFromContainer(container: Render, child: Instance) {
    if (child.type === "staff") {
      child.container.remove_staff(child.entity);
    }
  },
  removeChild(parent: Instance, child: Instance) {
    if (!parent || !child || parent.container !== child.container) {
      return;
    }

    if (parent.type === "staff") {
      child.container.remove_from_staff(parent.entity, child.entity);
    }
  },
  insertInContainerBefore(
    _container: Render,
    _child: Instance,
    _before: Instance
  ) {
    // TODO
  },
  insertBefore(parent: Instance, child: Instance, before: Instance) {
    if (parent.type === "staff") {
      parent.container.insert_to_staff_before(
        parent.entity,
        before.entity,
        child.entity
      );
    }
    if (parent.type === "bar" && child.type === "rnc") {
      parent.container.append_rnc(parent.entity, child.entity);
    }
  },

  prepareUpdate(
    _instance: Instance,
    _type,
    _oldProps,
    _newProps,
    _rootContainerInstance: Render,
    _currentHostContext
  ) {
    // TODO
  },
  commitUpdate(
    _instance: Instance,
    _updatePayload,
    _type,
    _oldProps,
    _newProps,
    _finishedWork
  ) {
    // TODO
  },

  commitMount(_instance: Instance, _type, _newProps, _finishedWork) {
    // TODO
  },

  finalizeInitialChildren() {
    return true;
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

export function render(whatToRender: any) {
  let container = Render.new();
  let newRoot = Reconciler.createContainer(container, false, false);

  Reconciler.updateContainer(whatToRender, newRoot, null, () => null);

  document.body.innerHTML = container.print_for_demo();
}
