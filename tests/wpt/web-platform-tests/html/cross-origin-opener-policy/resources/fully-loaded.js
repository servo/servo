// Return a promise, which resolves when new navigations aren't considered
// client-side redirects anymore.
//
// Note: A long `setTimeout` is used, because client-side redirect is an
// heuristic and isn't clearly specified.
function fullyLoaded() {
  return new Promise((resolve, reject) => {
    addEventListener('load', () => setTimeout(resolve, 2000))
  });
}
