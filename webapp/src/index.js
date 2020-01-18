Promise.all([import("../pkg/index.js"), import("react-reconciler")]).then(
  ([{ Barline, Render }, _ReactReconciler]) => {
    // console.log(ReactReconciler);
    // const reconciler = ReactReconciler.default({
    //   supportsMutation: true,
    //   createInstance(
    //     type,
    //     props,
    //     rootContainerInstance,
    //     hostContext,
    //     internalInstanceHandle
    //   ) {},
    //   createTextInstance(
    //     text,
    //     rootContainerInstance,
    //     hostContext,
    //     internalInstanceHandle
    //   ) {
    //     //
    //   },

    //   appendChildToContainer(container, child) {
    //     //
    //   },
    //   appendChild(parent, child) {
    //     //
    //   },
    //   appendInitialChild(parent, child) {
    //     //
    //   },

    //   removeChildFromContainer(container, child) {
    //     //
    //   },
    //   removeChild(parent, child) {
    //     //
    //   },
    //   insertInContainerBefore(container, child, before) {
    //     //
    //   },
    //   insertBefore(parent, child, before) {
    //     //
    //   },

    //   prepareUpdate(
    //     instance,
    //     type,
    //     oldProps,
    //     newProps,
    //     rootContainerInstance,
    //     currentHostContext
    //   ) {
    //     //
    //   },
    //   commitUpdate(
    //     instance,
    //     updatePayload,
    //     type,
    //     oldProps,
    //     newProps,
    //     finishedWork
    //   ) {
    //     //
    //   },

    //   finalizeInitialChildren() {},
    //   getChildHostContext() {},
    //   getPublicInstance() {},
    //   getRootHostContext() {},
    //   prepareForCommit() {},
    //   resetAfterCommit() {},
    //   shouldSetTextContent() {
    //     return false;
    //   }
    // });

    const render = Render.new();

    const staff = render.append_staff();
    const clef = render.create_clef();
    render.append_to_staff(staff, clef);
    document.body.innerHTML = render.print_for_demo(staff);

    setTimeout(() => {
      const bar1 = render.create_bar(4, 4);
      render.append_to_staff(staff, bar1);
      const rnc = render.create_rnc(-3, 0, [1, 4], true);
      render.append_rnc(bar1, rnc, [1, 4]);
      const barline = render.create_barline(Barline.Normal);
      render.append_to_staff(staff, barline);
      document.body.innerHTML = render.print_for_demo(staff);
    }, 500);

    let bar2;
    let rnc2;

    setTimeout(() => {
      bar2 = render.create_bar(4, 4);
      render.append_to_staff(staff, bar2);
      rnc2 = render.create_rnc(-3, 0, [1, 4], true);
      render.append_rnc(bar2, rnc2, [1, 4]);
      const barline = render.create_barline(Barline.Final);
      render.append_to_staff(staff, barline);
      document.body.innerHTML = render.print_for_demo(staff);
    }, 1000);

    setTimeout(() => {
      render.remove_rnc(bar2, rnc2);
      document.body.innerHTML = render.print_for_demo(staff);
    }, 1500);
  }
);
