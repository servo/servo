// receive token from the parent window, set cookie and notify parent
window.addEventListener("message", ({ origin, data }) => {
    if (origin !== "http://localhost:8000") {
        return;
    }

    document.cookie = `token=${data}; SameSite=Strict`;
    window.parent.postMessage("", "http://localhost:8000");
});
