const lineText = document.querySelector("#km-line-text");
if (lineText) {
    lineText.addEventListener("click", () => {
        window.location.href = "/";
    });
}

const yearElem = document.getElementById('km-year');
if (yearElem) {
    yearElem.textContent = new Date().getFullYear();
}

const backTop = document.getElementById('km-backtop');
if (backTop) {
    backTop.addEventListener('click', () => {
        window.scrollTo({ top: 0, behavior: 'smooth' });
    });
}