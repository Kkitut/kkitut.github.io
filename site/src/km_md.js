function km_md_setRender() {
    const renderer = {
        link({ href, title, text }) {
            const isLocal = href.startsWith('/');
            const arrow = !isLocal 
                ? `<img src="/res/arrow_from_braket.svg" class="linkarrow-icon">`
                : '';
            return `<a href="${href}" title="${title || ''}" ${isLocal ? '' : ' rel="noopener noreferrer"'} class="km-custom-link">${text}${arrow}</a>`;
        },
        image({ href, title, text }) {
            let style = '';
            if (title) {
                const match = title.match(/\^(\d+)/);
                if (match) {
                    const perc = match[1];
                    style = `width:${perc}%;height:${perc}%;`;
                    title = title.replace(match[0], '').trim();
                }
            }
            return `<img src="${href}" alt="${text}" title="${title || ''}" style="${style}" class="km-image">`;
        },
    };

    marked.use({ renderer });
}