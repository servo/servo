'use strict';

promise_test(async (t) => {
  let iframe = document.createElement('iframe');
  await new Promise(resolve => {
    iframe.src = '../resources/open-in-iframe.html';
    iframe.sandbox.add('allow-scripts');
    iframe.allow = 'usb';
    document.body.appendChild(iframe);
    iframe.addEventListener('load', resolve);
  });
  await new Promise(resolve => {
    window.addEventListener('message', t.step_func(messageEvent => {
      // The failure message of no device chosen is expected. The point here is
      // to validate not failing because of a sandboxed iframe.
      assert_equals(
          'FAIL: NotFoundError: Failed to execute \'requestDevice\' on ' +
              '\'USB\': No device selected.',
          messageEvent.data);
      resolve();
    }));
    iframe.contentWindow.postMessage('RequestDevice', '*');
  });
}, 'RequestDevice from a sandboxed iframe is valid.');
