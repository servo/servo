// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
//
// This is a regression test for crbug.com/583445. It checks an obscure bug in
// Chromium's handling of `document.open()` whereby the URL change would affect
// the document's origin after a javascript navigation.
//
// See also dcheng@'s comments on the original code review in which he
// introduced the precursor to this test:
// https://codereview.chromium.org/1675473002.

function nextMessage() {
  return new Promise((resolve) => {
    window.addEventListener("message", (e) => { resolve(e.data); }, {
      once: true
    });
  });
}

promise_test(async (t) => {
  // Embed a cross-origin frame A and set up remote code execution.
  const iframeA = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => { iframeA.remove(); });

  const uuidA = token();
  iframeA.src = remoteExecutorUrl(uuidA, { host: get_host_info().REMOTE_HOST });
  const ctxA = new RemoteContext(uuidA);

  // Frame A embeds a cross-origin frame B, which is same-origin with the
  // top-level frame. Frame B is the center of this test: it is where we will
  // verify that a bug does not grant it UXSS in frame A.
  //
  // Though we could reach into `iframeA.frames[0]` to get a proxy to frame B
  // and use `setTimeout()` like below to execute code inside it, we set up
  // remote code execution using `dispatcher.js` for better ergonomics.
  const uuidB = token();
  await ctxA.execute_script((url) => {
    const iframeB = document.createElement("iframe");
    iframeB.src = url;
    document.body.appendChild(iframeB);
  }, [remoteExecutorUrl(uuidB).href]);

  // Start listening for a message, which will come as a result of executing
  // the code below in frame B.
  const message = nextMessage();

  const ctxB = new RemoteContext(uuidB);
  await ctxB.execute_script(() => {
    // Frame B embeds an `about:blank` frame C.
    const iframeC = document.body.appendChild(document.createElement("iframe"));

    // We wish to execute code inside frame C, but it is important to this test
    // that its URL remain `about:blank`, so we cannot use `dispatcher.js`.
    // Instead we rely on `setTimeout()`.
    //
    // We use `setTimeout(string, ...)` instead of `setTimeout(function, ...)`
    // as the given script executes against the target window's global object
    // and does not capture any local variables.
    //
    // In order to have nice syntax highlighting and avoid quote-escaping hell,
    // we use a trick employed by `dispatcher.js`. We rely on the fact that
    // functions in JS have a stringifier that returns their source code. Thus
    // `"(" + func + ")()"` is a string that executes `func()` when evaluated.
    iframeC.contentWindow.setTimeout("(" + (() => {
      // This executes in frame C.

      // Frame C calls `document.open()` on its parent, which results in B's
      // URL being set to `about:blank` (C's URL).
      //
      // However, just before `document.open()` is called, B schedules a
      // self-navigation to a `javascript:` URL. This will occur after
      // `document.open()`, so the document will navigate from `about:blank` to
      // the new URL.
      //
      // This should not result in B's origin changing, so B should remain
      // same-origin with the top-level frame.
      //
      // Due to crbug.com/583445, this used to behave wrongly in Chromium. The
      // navigation code incorrectly assumed that B's origin should be inherited
      // from its parent A because B's URL was `about:blank`.
      //
      // It is important to schedule this from within the child, as this
      // guarantees that `document.open()` will be called before the navigation.
      // A previous version of this test scheduled this from within frame B
      // right after scheduling the call to `document.open()`, but that ran the
      // risk of races depending on which timeout fired first.
      parent.window.setTimeout("(" + (() => {
        // This executes in frame B.

        location = "javascript:(" + (() => {
          /* This also executes in frame B.
           *
           * Note that because this whole function gets stuffed in a JS URL,
           * single-line comments do not work, as they affect the following
           * lines. */

          let error;
          try {
            /* This will fail with a `SecurityError` if frame B is no longer
             * same-origin with the top-level frame. */
            top.window.testSameOrigin = true;
          } catch (e) {
            error = e;
          }

          top.postMessage({
            error: error?.toString(),
          }, "*");

        }) + ")()";

      }) + ")()", 0);

      // This executes in frame C.
      parent.document.open();

    }) + ")()", 0);
  });

  // Await the message from frame B after its navigation.
  const { error } = await message;
  assert_equals(error, undefined, "error accessing top frame from frame B");
  assert_true(window.testSameOrigin, "top frame testSameOrigin is mutated");

}, "Regression test for crbug.com/583445");

