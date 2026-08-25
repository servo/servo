// Helper for ariaNotify manual tests.
function tryCallAriaNotify(element, message, options = {}) {
  if (element.ariaNotify) {
    element.ariaNotify(message, options);
    return 'ariaNotify called' + ` with message: "${message}" and options: ${JSON.stringify(options)}`;
  } else {
    return 'the ariaNotify API is not supported in this browser';
  }
}
