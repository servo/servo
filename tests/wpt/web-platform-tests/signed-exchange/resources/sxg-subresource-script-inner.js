// Usually the alternate resource should have the same content as the original
// one (sxg-subresource-script.js), but for now we use differentiated content
// for easy testing.
window.parent.postMessage('from signed exchange', '*');
