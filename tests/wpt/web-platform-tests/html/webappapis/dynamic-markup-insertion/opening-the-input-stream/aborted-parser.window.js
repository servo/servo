// document.open() bails out early if there is an **active parser** with
// non-zero script nesting level. window.stop() aborts the current parser and
// makes it no longer active, and should allow document.open() to work.
// For more details, see https://bugzilla.mozilla.org/show_bug.cgi?id=1475000.

window.handlers = {};

async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  frame.src = "resources/aborted-parser-frame.html";
  window.handlers.afterOpen = t.step_func_done(() => {
    const openCalled = frame.contentDocument.childNodes.length === 0;
    frame.remove();
    assert_true(openCalled, "child document should be empty");
  });
}, "document.open() after parser is aborted");

// Note: This test should pass even if window.close() is not there, as
// document.open() is not executed synchronously in an inline script.
async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  frame.src = "resources/aborted-parser-async-frame.html";
  window.handlers.afterOpenAsync = t.step_func_done(() => {
    const openCalled = frame.contentDocument.childNodes.length === 0;
    frame.remove();
    assert_true(openCalled, "child document should be empty");
  });
}, "async document.open() after parser is aborted");
