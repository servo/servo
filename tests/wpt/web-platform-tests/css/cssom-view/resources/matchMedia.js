const IFRAME_BASE_WIDTH = "200";
const MEDIA_QUERY = `(max-width: ${IFRAME_BASE_WIDTH}px)`;

function createIframe(t) {
    const iframe = document.createElement("iframe");
    iframe.srcdoc = "";
    iframe.width = IFRAME_BASE_WIDTH;
    iframe.height = "100";
    iframe.style.border = "none";

    t.add_cleanup(() => {
        document.body.removeChild(iframe);
    });

    return new Promise(resolve => {
        iframe.addEventListener("load", () => {
            resolve(iframe);
        });

        document.body.appendChild(iframe);
    });
}

function triggerMQLEvent(iframe) {
    iframe.width = iframe.width === IFRAME_BASE_WIDTH ? "250" : IFRAME_BASE_WIDTH;
}

function waitForChangesReported() {
    return new Promise(resolve => {
        step_timeout(resolve, 75);
    });
}
