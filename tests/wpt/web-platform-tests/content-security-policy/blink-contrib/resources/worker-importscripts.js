try {
    importScripts("/content-security-policy/blink-contrib/resources/post-message.js");
    postMessage("importScripts allowed");
} catch (e) {
    postMessage("importScripts blocked: " + e);
}
