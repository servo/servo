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
      window.frameElement.remove();

      // This invokes the forEach() operator's internal observer's next steps,
      // which at least in Chromium, must have a special "context is detached"
      // check to early-return, so as to not crash.
      subscriber.next(1);
    });

    source.forEach(value => {
      parentResults.push(value);
    });
  `);

  // If we got here, we didn't crash! Let's also check that `results` is empty.
  assert_array_equals(results, []);
}, "forEach()'s internal observer's next steps do not crash in a detached document");

promise_test(async t => {
  const contentWin = await loadIframeAndReturnContentWindow();

  window.results = [];

  contentWin.eval(`
    const parentResults = parent.results;

    const source = new Observable(subscriber => {
      subscriber.next(1);
    });

    source.forEach(value => {
      window.frameElement.remove();
      parentResults.push(value);
    });
  `);

  assert_array_equals(results, [1]);
}, "forEach()'s internal observer's next steps do not crash when visitor " +
   "callback detaches the document");
