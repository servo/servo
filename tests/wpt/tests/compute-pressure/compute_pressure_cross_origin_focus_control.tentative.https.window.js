// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/resources/test-only-api.js
// META: script=resources/pressure-helpers.js

'use strict';

pressure_test(async (t, mockPressureService) => {
  const iframe = document.createElement('iframe');
  iframe.src = get_host_info().HTTPS_REMOTE_ORIGIN +
      '/compute-pressure/resources/support-iframe.html';
  const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
  document.body.appendChild(iframe);
  await iframeLoadWatcher.wait_for('load');
  iframe.contentWindow.focus();

  const observer = new PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });
  t.add_cleanup(() => {
    observer.disconnect();
    iframe.remove();
  });
  await observer.observe('cpu');
  mockPressureService.setPressureUpdate('cpu', 'critical');
  mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);

  return new Promise(resolve => t.step_timeout(resolve, 1000));
}, 'Observer in main frame should not receive PressureRecord when focused on cross-origin iframe');

pressure_test(async (t, mockPressureService) => {
  const iframe = document.createElement('iframe');
  iframe.src = get_host_info().HTTPS_REMOTE_ORIGIN +
      '/compute-pressure/resources/support-iframe.html';
  iframe.allow = 'compute-pressure';
  const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
  document.body.appendChild(iframe);
  await iframeLoadWatcher.wait_for('load');
  // Focus on the main frame to make the iframe lose focus, so that
  // PressureObserver in the iframe can't receive PressureRecord by default.
  // If the main frame has focus, but the iframe is cross-origin with the main
  // frame, PressureObserver in the iframe still can't receive PressureRecord.
  window.focus();

  return new Promise((resolve, reject) => {
    window.addEventListener('message', (e) => {
      if (e.data.result === 'timeout') {
        resolve();
      } else if (e.data.result === 'success') {
        reject('Observer should not receive PressureRecord');
      } else {
        reject('Got unexpected reply');
      }
    }, {once: true});
    iframe.contentWindow.postMessage({command: 'start'}, '*');
    mockPressureService.setPressureUpdate('cpu', 'critical');
    mockPressureService.startPlatformCollector(/*sampleRate=*/ 5.0);
  });
}, 'Observer in iframe should not receive PressureRecord when focused on cross-origin main frame');
