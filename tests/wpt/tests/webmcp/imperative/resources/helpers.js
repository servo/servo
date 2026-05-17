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
  if (actual.name !== expected.name) {
    return `names are unequal: ${actual.name} !== ${expected.name}`;
  }
  if (actual.description !== expected.description) {
    return `descriptions are unequal: ${actual.description} !== ${expected.description}`;
  }
  if (actual.inputSchema !== expected.inputSchema) {
    return `inputSchemas are unequal: ${actual.inputSchemas} !== ${expected.inputSchemas}`;
  }
  if (actual.origin !== expected.origin) {
    return `origins are unequal: ${actual.origin} !== ${expected.origin}`;
  }
  if (actual.annotations?.readOnlyHint !== expected.annotations?.readOnlyHint) {
    return `readOnlyHints are unequal: ${actual.annotations?.readOnlyHint} !== ${expected.annotations?.readOnlyHint}`;
  }
  if (actual.annotations?.untrustedContentHint !== expected.annotations?.untrustedContentHint) {
    return `untrustedContentHints are unequal: ${actual.annotations?.untrustedContentHint} !== ${expected.annotations?.untrustedContentHint}`;
  }

  return true;
}
