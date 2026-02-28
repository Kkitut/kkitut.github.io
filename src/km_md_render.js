async function km_md_render() {
    const out = document.getElementById("out");
    const res = await fetch("mdtextsrc.txt");
    let homesrc = await res.text();

    km_md_setRender();

    const raw = marked.parse(homesrc);
    out.innerHTML = raw;

    out.innerHTML = DOMPurify.sanitize(raw, {
        ADD_ATTR: ['class'],
    });
}

km_md_render();