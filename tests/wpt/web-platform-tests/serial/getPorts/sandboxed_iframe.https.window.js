'use strict';

let iframe = document.createElement('iframe');

promise_test(async () => {
  await new Promise(resolve => {
    iframe.src = '../resources/open-in-iframe.html';
    iframe.sandbox.add('allow-scripts');
    iframe.allow = 'serial';
    document.body.appendChild(iframe);
    iframe.addEventListener('load', resolve);
  });

  await new Promise(resolve => {
    iframe.contentWindow.postMessage({type: 'GetPorts'}, '*');

    window.addEventListener('message', (messageEvent) => {
      assert_equals('Success', messageEvent.data);
      resolve();
    });
  });
}, 'GetPorts from a sandboxed iframe is valid.');
