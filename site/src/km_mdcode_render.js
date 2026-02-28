async function km_mdcode_render() {
    const out = document.getElementById("out");
    const res = await fetch("mdtextsrc.txt");
    let homesrc = await res.text();

    km_md_setRender();
    km_mdcode_setOption();

    const raw = marked.parse(homesrc);
    out.innerHTML = raw;

    out.innerHTML = DOMPurify.sanitize(raw, {
        ADD_ATTR: ['class'],
    });

    km_mdcode_apply();
}

km_mdcode_render();