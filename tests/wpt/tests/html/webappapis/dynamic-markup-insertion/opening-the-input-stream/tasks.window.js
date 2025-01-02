// An older version of the HTML Standard mandated that document.open() remove
// all tasks associated with the document on which open() is called. This step
// has been proposed to be removed. This series of tests ensures that this step
// is no longer executed.
//
// This file comprehensively (but not exhaustively) tests for many queued tasks
// that may be observable. Each taskTest() call in fact runs two tests: the
// first one "tasks without document.open()" does not actually run
// document.open(), just to test that the tested task works ordinarily; the
// second actually calls document.open() to test if the method call removes
// that specific task from the queue.

// This is necessary to allow the promise rejection test below.
setup({
  allow_uncaught_exception: true
});

function taskTest(description, testBody) {
  async_test(t => {
    const frame = document.body.appendChild(document.createElement("iframe"));
    // The empty HTML seems to be necessary to cajole Chrome and Safari into
    // firing a load event asynchronously, which is necessary to make sure the
    // frame's document doesn't have a parser associated with it.
    // See: https://crbug.com/569511
    frame.src = "/common/blank.html";
    t.add_cleanup(() => frame.remove());
    frame.onload = t.step_func(() => {
      // Make sure there is no parser. Firefox seems to have an additional
      // non-spec-compliant readiness state "uninitialized", so test for the
      // two known valid readiness states instead.
      // See: https://bugzilla.mozilla.org/show_bug.cgi?id=1191683
      assert_in_array(frame.contentDocument.readyState, ["interactive", "complete"]);
      testBody(t, frame, doc => {});
    });
  }, `tasks without document.open() (${description})`);

  async_test(t => {
    const frame = document.body.appendChild(document.createElement("iframe"));
    // The empty HTML seems to be necessary to cajole Chrome into firing a load
    // event, which is necessary to make sure the frame's document doesn't have
    // a parser associated with it.
    // See: https://crbug.com/569511
    frame.src = "/common/blank.html";
    t.add_cleanup(() => frame.remove());
    frame.onload = t.step_func(() => {
      // Make sure there is no parser. Firefox seems to have an additional
      // non-spec-compliant readiness state "uninitialized", so test for the
      // two known valid readiness states instead.
      // See: https://bugzilla.mozilla.org/show_bug.cgi?id=1191683
      assert_in_array(frame.contentDocument.readyState, ["interactive", "complete"]);
      testBody(t, frame, doc => doc.open());
    });
  }, `document.open() and tasks (${description})`);
}

taskTest("timeout", (t, frame, open) => {
  frame.contentWindow.setTimeout(t.step_func_done(), 100);
  open(frame.contentDocument);
});

taskTest("window message", (t, frame, open) => {
  let counter = 0;
  frame.contentWindow.postMessage(undefined, "*");
  open(frame.contentDocument);
  frame.contentWindow.postMessage(undefined, "*");
  frame.contentWindow.onmessage = t.step_func(e => {
    assert_equals(e.data, undefined);
    counter++;
    assert_less_than_equal(counter, 2);
    if (counter == 2) {
      t.done();
    }
  });
});

taskTest("canvas.toBlob()", (t, frame, open) => {
  const canvas = frame.contentDocument.body.appendChild(frame.contentDocument.createElement("canvas"));
  canvas.toBlob(t.step_func_done());
  open(frame.contentDocument);
});

taskTest("MessagePort", (t, frame, open) => {
  frame.contentWindow.eval(`({ port1, port2 } = new MessageChannel());`);
  frame.contentWindow.port2.onmessage = t.step_func_done(ev => {
    assert_equals(ev.data, "Hello world");
  });
  frame.contentWindow.port1.postMessage("Hello world");
  open(frame.contentDocument);
});

taskTest("Promise rejection", (t, frame, open) => {
  // There is currently some ambiguity on which Window object the
  // unhandledrejection event should be fired on. Here, let's account for that
  // ambiguity and allow event fired on _any_ global to pass this test.
  // See:
  // - https://github.com/whatwg/html/issues/958,
  // - https://bugs.webkit.org/show_bug.cgi?id=187822
  const promise = frame.contentWindow.eval("Promise.reject(42);");
  open(frame.contentDocument);
  const listener = t.step_func_done(ev => {
    assert_equals(ev.promise, promise);
    assert_equals(ev.reason, 42);
  });
  frame.contentWindow.onunhandledrejection = listener;
  window.onunhandledrejection = listener;
});
