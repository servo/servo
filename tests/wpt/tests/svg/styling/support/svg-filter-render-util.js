function navigateAndWaitForLoad(iframeElement, newUrl) {
    return new Promise((resolve) => {
        iframeElement.addEventListener('load', () => resolve(), { once: true });
        iframeElement.contentWindow.location.href = newUrl;
    });
}

function waitFrame() {
    return new Promise((resolve) => {
        requestAnimationFrame(resolve)
    });
}

async function waitForRender(callback) {
    await waitFrame();
    await waitFrame();
    callback();
}
