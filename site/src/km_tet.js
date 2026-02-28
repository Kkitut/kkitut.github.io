const km_tet_colorMap  = {
    'i':'#3abadaff','j':'#3c77c4ff','l':'#e29118ff',
    'o':'#e0c83cff','s':'#4bc555ff','t':'#b92ab2ff',
    'z':'#db5555ff','g':'#a0a0a0ff'
};

const km_tet_miniShapes = {
    'i': [[0,0,0,0],[1,1,1,1]],
    'j': [[1,0,0],[1,1,1]],
    'l': [[0,0,1],[1,1,1]],
    'o': [[1,1],[1,1]],
    's': [[0,1,1],[1,1,0]],
    't': [[0,1,0],[1,1,1]],
    'z': [[1,1,0],[0,1,1]]
};

function km_tet_renderMiniMino(char, heightPx) {
    if (!char) return '';

    const shape = km_tet_miniShapes[char];
    if (!shape) return '';

    const rows = shape.length;
    const cols = shape[0].length;
    const cellSize = heightPx / rows;

    const canvas = document.createElement('canvas');
    canvas.width = Math.ceil(cols * cellSize);
    canvas.height = heightPx;

    const ctx = canvas.getContext('2d');
    const color = km_tet_colorMap[char] || '#ffffff';

    for (let y = 0; y < rows; y++) {
        for (let x = 0; x < cols; x++) {
            if (shape[y][x]) {
                ctx.fillStyle = color;
                ctx.fillRect(
                    Math.floor(x * cellSize),
                    Math.floor(y * cellSize),
                    Math.ceil(cellSize),
                    Math.ceil(cellSize)
                );
            }
        }
    }

    return canvas.toDataURL();
}

const km_tet_miniCache = {};
['i','j','l','o','s','t','z','g'].forEach(char => {
    km_tet_miniCache[char] = km_tet_renderMiniMino(char, 16);
});

function km_tet_convertMiniMinoTags(text, mirror = false) {
    return text.replace(/\[\[([ijlostzg])(?:s(\d+))?\]\]/gi, (match, char, sizeStr) => {
        const heightPx = sizeStr ? parseInt(sizeStr) : 16;

        if (mirror) {
            const swap = { j:'l', l:'j', s:'z', z:'s' };
            const lower = char.toLowerCase();
            if (swap[lower]) char = swap[lower];
        }

        const dataUrl = heightPx === 16 ? km_tet_miniCache[char] : km_tet_renderMiniMino(char, heightPx);
        return `<img src="${dataUrl}" class="km-minimino" alt="${char.toUpperCase()}" style="height:${heightPx}px;width:auto;">`;
    });
}

function km_tet_lightenColor(hex, lightAmount = 0.2, satAmount = 1) {
    let r = parseInt(hex.slice(1,3),16) / 255;
    let g = parseInt(hex.slice(3,5),16) / 255;
    let b = parseInt(hex.slice(5,7),16) / 255;

    let max = Math.max(r, g, b), min = Math.min(r, g, b);
    let h, s, l = (max + min) / 2;

    if (max === min) {
        r = parseInt(hex.slice(1,3),16);
        g = parseInt(hex.slice(3,5),16);
        b = parseInt(hex.slice(5,7),16);

        lightAmount *= 2.4;
        
        r = Math.min(255, Math.floor(r + (255 - r) * lightAmount));
        g = Math.min(255, Math.floor(g + (255 - g) * lightAmount));
        b = Math.min(255, Math.floor(b + (255 - b) * lightAmount));

        return `rgba(${r},${g},${b},1)`;
    } else {
        let d = max - min;
        s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
        switch(max){
            case r: h = (g - b) / d + (g < b ? 6 : 0); break;
            case g: h = (b - r) / d + 2; break;
            case b: h = (r - g) / d + 4; break;
        }
        h /= 6;
    }

    l = Math.min(1, l + lightAmount);
    s = Math.min(1, s + satAmount);

    function hue2rgb(p, q, t){
        if(t < 0) t += 1;
        if(t > 1) t -= 1;
        if(t < 1/6) return p + (q - p) * 6 * t;
        if(t < 1/2) return q;
        if(t < 2/3) return p + (q - p) * (2/3 - t) * 6;
        return p;
    }

    let r1, g1, b1;
    if(s === 0){
        r1 = g1 = b1 = l;
    } else {
        let q = l < 0.5 ? l * (1 + s) : l + s - l * s;
        let p = 2 * l - q;
        r1 = hue2rgb(p, q, h + 1/3);
        g1 = hue2rgb(p, q, h);
        b1 = hue2rgb(p, q, h - 1/3);
    }

    r1 = Math.round(r1 * 255);
    g1 = Math.round(g1 * 255);
    b1 = Math.round(b1 * 255);

    return `rgba(${r1},${g1},${b1},1)`;
}

function km_tet_convertTetrisTagsToImages(text, mirror = false) {
    return text.replace(/@\[tt:([^\]]+)\]/g, (match, spec) => {
        const m = spec.match(/^(\d+)x(\d+)(s(\d+))?;([ijlostzgIJLOSTZG0* \-]+)$/);
        if (!m) return match;

        const width = parseInt(m[1]);
        const height = parseInt(m[2]);
        const size = m[4] ? parseInt(m[4]) : 22;
        let layoutRaw = m[5];

        if (layoutRaw[0] === '-') {
            const cellSize = Math.round(size);
            const glowHeight = Math.round(cellSize * 0.2);

            const canvas = document.createElement('canvas');
            canvas.width = width * cellSize;
            canvas.height = height * cellSize + glowHeight;

            return `<img src="${canvas.toDataURL()}" class="km-tetromino">`;
        }

        let idx = 0;
        const lines = [];
        for (let y = 0; y < height; y++) {
            let brighten = false;
            if (layoutRaw[idx] === '*') { brighten = true; idx++; }
            const line = [];
            for (let x = 0; x < width; x++) {
                const cell = layoutRaw[idx++] || '0';
                line.push({ cell, brighten });
            }
            lines.push(line);
        }

        if (mirror) {
            const swap = { j:'l', l:'j', s:'z', z:'s', J:'L', L:'J', S:'Z', Z:'S' };
            for (let y = 0; y < lines.length; y++) {
                lines[y] = lines[y].reverse().map(({cell, brighten}) => ({
                    cell: swap[cell] ?? cell,
                    brighten
                }));
            }
        }

        const cellSize = Math.round(size);
        const glowHeight = Math.round(cellSize * 0.2);
        const canvas = document.createElement('canvas');
        canvas.width = width * cellSize;
        canvas.height = height * cellSize + glowHeight;
        const ctx = canvas.getContext('2d');
        ctx.imageSmoothingEnabled = false;

        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                const { cell, brighten } = lines[height - 1 - y][x];
                if (cell === '0') continue;

                let color = km_tet_colorMap[cell.toLowerCase()] || '#ffffff';
                if (brighten) color = km_tet_lightenColor(color, 0.22);

                const px = x * cellSize;
                const py = y * cellSize + glowHeight;
                ctx.fillStyle = color;
                ctx.fillRect(px, py, cellSize, cellSize);

                if (cell === cell.toUpperCase() && cell !== '0') {
                    const dotSize = Math.max(1, Math.floor(cellSize * 0.5));
                    ctx.strokeStyle = '#ffffff';
                    ctx.lineWidth = 2.8;
                    ctx.beginPath();
                    ctx.arc(px + cellSize/2, py + cellSize/2, dotSize/2, 0, Math.PI*2);
                    ctx.stroke();
                }
            }
        }

        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                const { cell, brighten } = lines[height - 1 - y][x];
                if (cell === '0') continue;

                const aboveCell = y === 0 ? '0' : lines[height - 1 - (y-1)][x].cell || '0';
                if (aboveCell === '0') {
                    const baseColor = km_tet_colorMap[cell.toLowerCase()] || '#ffffff';
                    ctx.fillStyle = brighten ? km_tet_lightenColor(baseColor, 0.32) : km_tet_lightenColor(baseColor);
                    ctx.fillRect(x * cellSize, y * cellSize, cellSize, glowHeight);
                }
            }
        }

        return `<img src="${canvas.toDataURL()}" class="km-tetromino">`;
    });
}

function km_tet_applyTetAni(container) {
    const containers = typeof container === 'string' 
        ? new DOMParser().parseFromString(container, 'text/html').body 
        : container;

    containers.querySelectorAll("km-tetani").forEach(container => {
        const imgs = [...container.querySelectorAll("img.km-tetromino")];
        if (imgs.length > 1) {
            imgs.forEach((img, i) => img.style.display = i === 0 ? "inline" : "none");

            let current = 0;
            const speed = parseInt(container.getAttribute('speed')) || 2400;

            setInterval(() => {
                imgs[current].style.display = "none";
                current = (current + 1) % imgs.length;
                imgs[current].style.display = "inline";
            }, speed);
        }
    });

    return containers;
}