import('../pkg/index.js').then(({Barline, Render}) => {
    const render = Render.new();

    const staff_entity = render.append_staff();
    render.append_clef(staff_entity);
    document.body.innerHTML = render.print_for_demo(staff_entity);

    setTimeout(() => {

    const bar1_entity = render.append_bar(staff_entity, 4, 4);
        render.append_rnc(bar1_entity, -3, 0, [1, 4], true);
        render.append_barline(staff_entity, Barline.Normal);
        document.body.innerHTML = render.print_for_demo(staff_entity);
    }, 500);

    setTimeout(() => {
        const bar2_entity = render.append_bar(staff_entity, 4, 4);
        render.append_rnc(bar2_entity, -3, 0, [1, 4], true);
        render.append_barline(staff_entity, Barline.Final);
        document.body.innerHTML = render.print_for_demo(staff_entity);
    }, 1000);
});

