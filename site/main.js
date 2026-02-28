const middleElement = document.querySelector('#middle-logo');

let targetRotateX = 0, targetRotateY = 0;
let currentRotateX = 0, currentRotateY = 0;

const maxRotateX = 28;
const maxRotateY = 28;
const ease = 0.04;

function updateMiddlePosition() {
    const windowHeight = window.innerHeight;
    const windowWidth = window.innerWidth;

    const newTop = windowHeight / 2 + window.scrollY / 1.4;
    const newLeft = windowWidth / 2;

    middleElement.style.position = 'absolute';
    middleElement.style.top = `${newTop}px`;
    middleElement.style.left = `${newLeft}px`;
}

function handleResize() {
    updateMiddlePosition();
}

function handleMouseMove(e) {
    const rect = middleElement.getBoundingClientRect();
    const centerX = rect.left + rect.width / 2;
    const centerY = rect.top + rect.height / 2;

    const deltaX = e.clientX - centerX;
    const deltaY = e.clientY - centerY;

    targetRotateY = Math.max(Math.min(deltaX / rect.width * maxRotateY, maxRotateY), -maxRotateY);
    targetRotateX = Math.max(Math.min(-deltaY / rect.height * maxRotateX, maxRotateX), -maxRotateX);
}

function handleMouseLeave() {
    targetRotateX = 0;
    targetRotateY = 0;
}

function animate() {
    currentRotateX += (targetRotateX - currentRotateX) * ease;
    currentRotateY += (targetRotateY - currentRotateY) * ease;

    middleElement.style.transform = `
        translate(-50%, -50%)
        rotateX(${currentRotateX}deg)
        rotateY(${currentRotateY}deg)
    `;

    requestAnimationFrame(animate);
}

updateMiddlePosition();
animate();

window.addEventListener('resize', handleResize);
window.addEventListener('scroll', handleResize);
window.addEventListener('mousemove', handleMouseMove);
window.addEventListener('mouseout', handleMouseLeave);