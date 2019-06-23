try {
    importScripts("/content-security-policy/support/post-message.js");
} catch (e) {
    postMessage("importScripts blocked");
}
