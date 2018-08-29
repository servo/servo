// Historically, document.open() created an entry in the session history so
// that the original page could be seen by going back. Test that this behavior
// no longer occurs.
//
// This test uses window.open() for variety, as most other tests in this
// directory use document.open(). An <iframe> would probably work also. We can
// always add an <iframe>-based test later if it is deemed necessary.

const t = async_test("document.open should not add an entry to the session history");

const frameURL = new URL("resources/history-frame.html", document.URL).href;

let origLength;
window.onFrameLoaded = t.step_func(() => {
  window.onFrameLoaded = t.unreached_func("onFrameLoaded should only be called once");
  assert_equals(win.document.URL, frameURL);
  assert_true(win.document.body.textContent.includes("Old"));
  origLength = win.history.length;
});
window.onDocumentOpen = t.step_func_done(() => {
  window.onDocumentOpen = t.unreached_func("onDocumentOpen should only be called once");
  assert_equals(win.document.URL, frameURL);
  assert_true(win.document.body.textContent.includes("New"));
  assert_not_equals(origLength, undefined);
  assert_equals(win.history.length, origLength);
});

const win = window.open(frameURL);
t.add_cleanup(() => win.close());
