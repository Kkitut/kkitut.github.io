async function km_mdtetsel_render() {
    const out = document.getElementById("out");
    const res = await fetch("mdtextsrc.txt");
    let homesrc = await res.text();

    homesrc = km_tet_convertMiniMinoTags(homesrc);
    homesrc = km_tet_convertTetrisTagsToImages(homesrc);

    km_md_setRender();

    const raw = marked.parse(homesrc);
    out.innerHTML = raw;

    out.innerHTML = DOMPurify.sanitize(raw, {
        ADD_TAGS: ['km-tetani'],
        ADD_ATTR: ['class', 'speed'],
    });

    km_tet_applyTetAni(out);
    km_section_buildSectionTab(out);
}

km_mdtetsel_render();