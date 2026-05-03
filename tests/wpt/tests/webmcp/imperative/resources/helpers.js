// Helper to wait for postMessage response from iframe
function waitForIframeMessage(action) {
  return new Promise(resolve => {
    const listener = e => {
      if (e.data && e.data.action === action) {
        window.removeEventListener('message', listener);
        resolve(e.data);
      }
    };
    window.addEventListener('message', listener);
  });
}

// Helper to compare tool objects that should conform to the `RegisteredTool`
// dictionary.
function toolsAreEqual(actual, expected) {
  if (actual.name !== expected.name) return false;
  if (actual.description !== expected.description) return false;
  if (actual.inputSchema !== expected.inputSchema) return false;
  return true;
}
