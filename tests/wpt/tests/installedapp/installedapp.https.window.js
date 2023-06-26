// META: script=resources/utils.js

'use strict';

// https://wicg.github.io/get-installed-related-app/spec

promise_test(async t => {

  assert_true('getInstalledRelatedApps' in navigator);
  assert_array_equals(await navigator.getInstalledRelatedApps(), []);

}, 'Check calling getInstalledRelatedApps works as expected');

promise_test(async t => {

  const iframeWindow = await new Promise(resolve => {
    const iframe = document.createElement('iframe');
    iframe.src = 'resources/iframe.html';
    iframe.onload = () => resolve(iframe.contentWindow);
    document.body.appendChild(iframe);
  });

  try {
    await iframeWindow.navigator.getInstalledRelatedApps();
    assert_unreached('expected a DOMException, but none was thrown');
  } catch (e) {
    assert_equals(e.name, 'InvalidStateError');
  }

}, 'Calling getInstalledrelatedApps from an iframe fails');