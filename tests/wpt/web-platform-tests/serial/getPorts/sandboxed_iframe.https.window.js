'use strict';

promise_test(async (t) => {
  let iframe = document.createElement('iframe');
  await new Promise(resolve => {
    iframe.src = '../resources/open-in-iframe.html';
    iframe.sandbox.add('allow-scripts');
    iframe.allow = 'serial';
    document.body.appendChild(iframe);
    iframe.addEventListener('load', resolve);
  });

  await new Promise(resolve => {
    window.addEventListener('message', t.step_func(messageEvent => {
      assert_equals(messageEvent.data, 'Success');
      resolve();
    }));
    iframe.contentWindow.postMessage({type: 'GetPorts'}, '*');
  });
}, 'GetPorts from a sandboxed iframe is valid.');
