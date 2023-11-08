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
  // Hang this off of the main document's global, so the child can easily reach
  // it.
  window.results = [];
  const contentWin = await loadIframeAndReturnContentWindow();

  contentWin.eval(`
    // Get a reference to the parent result array before we detach and lose
    // access to the parent.
    const parentResults = parent.results;

    const source = new Observable((subscriber) => {
      parentResults.push("subscribe");
      // Detach the iframe and push a value to the subscriber/Observer.
      window.frameElement.remove();
      parentResults.push("detached");
      subscriber.next("next");
      subscriber.complete();
      subscriber.error("error");
    });
    source.subscribe({
      next: v => {
        // Should never run.
        parentResults.push(v);
      },
      complete: () => {
        // Should never run.
        parentResults.push("complete");
      },
      erorr: e => {
        // Should never run.
        parentResults.push(e);
      }
    });
  `);

  assert_array_equals(results, ["subscribe", "detached"]);
}, "No observer handlers can be invoked in detached document");

promise_test(async t => {
  const contentWin = await loadIframeAndReturnContentWindow();

  // Set a global error handler on the iframe document's window, and verify that
  // it is never called (because the thing triggering the error happens when the
  // document is detached, and "reporting the exception" relies on an attached
  // document).
  contentWin.addEventListener("error",
      t.unreached_func("Error should not be called"), { once: true });

  contentWin.eval(`
    const source = new Observable((subscriber) => {
      // Detach the iframe and push an error, which would normally "report the
      // exception", since this subscriber did not specify an error handler.
      window.frameElement.remove();
      subscriber.error("this is an error that should not be reported");
    });
    source.subscribe();
  `);
}, "Subscriber.error() does not \"report the exception\" even when an " +
   "`error()` handler is not present, when it is invoked in a detached document");

promise_test(async t => {
  // Make this available off the global so the child can reach it.
  window.results = [];
  const contentWin = await loadIframeAndReturnContentWindow();

  // Set a global error handler on the iframe document's window, and verify that
  // it is never called (because the thing triggering the error happens when the
  // document is detached, and "reporting the exception" relies on an attached
  // document).
  contentWin.addEventListener("error",
      t.unreached_func("Error should not be called"), { once: true });

  contentWin.eval(`
    const parentResults = parent.results;
    const source = new Observable((subscriber) => {
      // This should never run.
      parentResults.push('subscribe');
    });

    // Detach the iframe and try to subscribe.
    window.frameElement.remove();
    parentResults.push('detached');
    source.subscribe();
  `);

  assert_array_equals(results, ["detached"], "Subscribe callback is never invoked");
}, "Cannot subscribe to an Observable in a detached document");
