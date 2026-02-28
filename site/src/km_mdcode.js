function km_mdcode_setOption() {
    marked.setOptions({
        langPrefix: 'language-',
        highlight: function(code, lang) {
        const language = Prism.languages[lang] || Prism.languages.markup;
        return Prism.highlight(code, language, lang);
        },
    });
}

function km_mdcode_apply() {
    Prism.highlightAll();
    
    const codeBlocks = out.querySelectorAll('pre');
    codeBlocks.forEach(pre => {
        pre.style.position = 'relative';
        pre.style.paddingTop = '32px';

        const codeEl = pre.querySelector('code');

        const topBar = document.createElement('div');
        topBar.className = 'km-code-top-bar';

        const langName = document.createElement('span');
        langName.className = 'km-code-lang-name';
        const langClass = [...codeEl.classList].find(c => c.startsWith('language-'));
        langName.textContent = langClass ? langClass.replace('language-', '') : 'TEXT';

        const btn = document.createElement('img');
        btn.className = 'km-code-copy-btn';
        btn.src = '/res/copy.svg';
        btn.alt = 'copy';
            
        btn.addEventListener('click', () => {
            navigator.clipboard.writeText(codeEl.innerText).then(() => {
                btn.src = '/res/check.svg';
                setTimeout(() =>{
                    btn.src = '/res/copy.svg';
                }, 1200);
            });
        });

        topBar.appendChild(langName);
        topBar.appendChild(btn);

        pre.prepend(topBar);
    });
}