const params = new URLSearchParams(location.search);

// Take a key used for storing a test result in the server.
const key = params.get('key');

// Take a target hint to decide a target context for prerendering.
const rule_extras = {'target_hint': getTargetHint()};

// Speculation rules injection is blocked in the csp-script-src 'self' test.
const block = location.pathname.endsWith('csp-script-src-self.html');

// The main test page (csp-script-src-*.html) in the parent directory) will load
// this page only with the "key" parameter. This page will then try prerendering
// itself with the "run-test" parameter. When "run-test" is in the URL we'll
// actually start the test process and record the results to send back to the
// main test page. We do this because the main test page cannot navigate itself
// but it also cannot open a popup to a prerendered browsing context so the
// prerender triggering and activation must both happen in this popup.
const run_test = params.has('run-test');
if (!run_test) {
  // Generate a new stash key so we can communicate with the prerendered page
  // about when to close the popup.
  const done_key = token();
  const url = new URL(document.URL);
  url.searchParams.append('run-test', '');
  url.searchParams.append('done-key', done_key);

  if (block) {
    // Observe `securitypolicyviolation` event that will be triggered by
    // startPrerendering().
    document.addEventListener('securitypolicyviolation', e => {
      if (e.effectiveDirective != 'script-src' &&
          e.effectiveDirective != 'script-src-elem') {
        const message = 'unexpected effective directive: ' + e.effectiveDirective;
        writeValueToServer(key, message).then(() => { window.close(); });
      } else {
        const message = 'blocked by ' + e.effectiveDirective;
        writeValueToServer(key, message).then(() => { window.close(); });
      }
    });
  }

  startPrerendering(url.toString(), rule_extras);

  // Wait until the prerendered page signals us it's ready to close.
  nextValueFromServer(done_key).then(() => {
    window.close();
  });
} else {
  if (block) {
    writeValueToServer(key, 'unexpected prerendering');
  } else {
    // Tell the harness the initial document.prerendering value.
    writeValueToServer(key, document.prerendering);

    // Tell the prerendering initiating page test being finished.
    const done_key = params.get('done-key');
    writeValueToServer(done_key, "done");
  }
}
