// META: title=Top-level navigation tests with cross origin & user activated child frames
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-actions.js
// META: script=/resources/testdriver-vendor.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js
// META: script=./resources/sandbox-top-navigation-helper.sub.js

'use strict';

promise_test(
    async t => {
      const main = await setupTest();

      const iframe = await createNestedIframe(main, 'HTTP_ORIGIN', '', '');
      await activate(iframe);

      const new_iframe = await navigateFrameTo(iframe, 'HTTP_REMOTE_ORIGIN');
      await attemptTopNavigation(new_iframe, true);
    },
    'A same-site unsandboxed iframe navigation does not consume user ' +
        'activation and allows top-level navigation.');
