let mirror = false;

async function km_mdtetselmirror_render() {
    const out = document.getElementById("out");
    const res = await fetch("mdtextsrc.txt");
    let homesrc = await res.text();

    homesrc = km_tet_convertMiniMinoTags(homesrc, mirror);
    homesrc = km_tet_convertTetrisTagsToImages(homesrc, mirror);

    km_md_setRender();

    const raw = marked.parse(homesrc);

    out.innerHTML = '';

    out.innerHTML = DOMPurify.sanitize(raw, {
        ADD_TAGS: ['km-tetani'],
        ADD_ATTR: ['class', 'speed'],
    });

    km_tet_applyTetAni(out);
    km_section_buildSectionTab(out);
}

document.getElementById("km-tetmirror").addEventListener("click", () => {
    mirror = !mirror;
    km_mdtetselmirror_render();
});

km_mdtetselmirror_render();