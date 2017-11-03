try {
    importScripts("/content-security-policy/support/post-message.js");
    postMessage("importScripts allowed");
} catch (e) {
    postMessage("importScripts blocked");
}
