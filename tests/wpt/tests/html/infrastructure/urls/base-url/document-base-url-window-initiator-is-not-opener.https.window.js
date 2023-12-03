// Tests that a popup about:blank window inherits its base url from
// the initiator, and not the opener.
const runTest = (description) => {
  const opener_base_uri = document.baseURI;

  promise_test((test) => {
    return new Promise(async resolve => {
      window.popup = window.open();
      test.add_cleanup(() => popup.close());
      assert_equals(window.popup.location.href, 'about:blank');

      // Create iframe to be the initiator.
      const iframe = document.createElement('iframe');
      iframe.srcdoc = `
      <head>
      <base href='https://example.com'>
      <script>
        window.top.popup.location.href = 'about:blank';
      </scr` + `ipt>
      </head>
      <body></body>
      `;

      const popup_navigated = new Promise(r => window.popup.onpagehide = e => r());
      document.body.append(iframe);
      await popup_navigated;  // This makes sure the old child has unloaded, but
                             // with the timeout below it's really not needed.

      // This is necessary, or else the test times out. The about:blank load
      // does not fire an onload event we can access.
      test.step_timeout(resolve, 500);
    }).then(() => {
      assert_equals('https://example.com/', window.popup.document.baseURI);
    });
  }, description);
};

onload = () => {
  runTest("window.open() gets base url from initiator not opener.");
};
