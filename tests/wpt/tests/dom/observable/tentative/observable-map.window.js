async function loadIframeAndReturnContentWindow() {
   // Create and attach an iframe.
  const iframe = document.createElement('iframe');
  const iframeLoadPromise = new Promise((resolve, reject) => {
    iframe.onload = resolve;
    iframe.onerror = reject;
  });
  document.body.append(iframe);
  await iframeLoadPromise;
  return iframe.contentWindow;
}

promise_test(async t => {
  const contentWin = await loadIframeAndReturnContentWindow();

  window.results = [];

  contentWin.eval(`
    const parentResults = parent.results;

    const source = new Observable(subscriber => {
      // Detach the document before calling next().
      window.frameElement.remove();

      // This invokes the map() operator's internal observer's next steps,
      // which at least in Chromium, must have a special "context is detached"
      // check to early-return, so as to not crash before invoking the "mapper"
      // callback supplied to the map() operator.
      subscriber.next(1);
    });

    source.map(value => {
      parentResults.push(value);
    }).subscribe(v => parentResults.push(v));
  `);

  // If we got here, we didn't crash! Let's also check that `results` is empty.
  assert_array_equals(results, []);
}, "map()'s internal observer's next steps do not crash in a detached document");

