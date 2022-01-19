// Returns a promise that asserts the "load" and "pageshow" events are not
// fired on |target|.
function assertNoLoadAndPageshowEvent(t, target) {
  target.addEventListener("load", t.unreached_func("load should not be fired"));
  target.addEventListener("pageshow", t.unreached_func("pageshow should not be fired"));
  return new Promise(resolve => {
    // Wait 50ms to ensure events fired after asynchronous navigations are
    // also captured.
    setTimeout(resolve, 50);
  });
}
