function km_section_buildSectionTab(container) {
    const headings = container.querySelectorAll("h1, h2, h3"); 
    if (!headings.length) return;

    const tab = document.getElementById("km-section-tab");
    if (!tab) return;

    tab.innerHTML = '';

    const ul = document.createElement("ul");
    headings.forEach((h, i) => {
        if (!h.id) h.id = "km-section-" + i;

        const li = document.createElement("li");
        li.dataset.level = h.tagName.slice(1);

        const a = document.createElement("a");
        a.href = "#" + h.id;
        a.textContent = h.textContent;

        li.appendChild(a);
        ul.appendChild(li);
    });
    tab.appendChild(ul);
}