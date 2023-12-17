
async function runNavigateAncestorTest(test_type, ancestor_type) {
  // See documentation in `resources/navigate-ancestor-test-runner.https.html`.
  // For each test type here, this document opens a new auxiliary window that
  // runs the actual test. The tests in some way or another, direct a frame
  // *inside* a fenced frame to navigate an ancestor frame via an
  // <a target="_parent|_top"></a>. We need to run the real test in a new window
  // so that if that window ends up navigating unexpectedly (because the fenced
  // frame can accidentally navigated its embedder, for example) we can detect
  // it from ths page, which never navigates away.
  const navigate_ancestor_key = token();
  const navigate_ancestor_from_nested_key = token();

  const win = window.open(generateURL(
      "resources/navigate-ancestor-test-runner.https.html",
      [navigate_ancestor_key, navigate_ancestor_from_nested_key]));
  await new Promise(resolve => {
    win.onload = resolve;
  });

  const pagehidePromise = new Promise(resolve => {
    win.onpagehide = resolve;
  });

  await win.runTest(test_type, ancestor_type);
  win.close();
  await pagehidePromise;
}
