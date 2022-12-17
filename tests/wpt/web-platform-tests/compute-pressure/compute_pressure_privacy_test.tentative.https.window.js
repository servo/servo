// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/media.js
// META: script=/mediacapture-streams/permission-helper.js
// META: script=/picture-in-picture/resources/picture-in-picture-helpers.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

'use strict';

promise_test(async t => {
  const video = await loadVideo();
  document.body.appendChild(video);
  const pipWindow = await requestPictureInPictureWithTrustedClick(video);
  assert_not_equals(pipWindow.width, 0);
  assert_not_equals(pipWindow.height, 0);

  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  // Focus on the iframe to make the main frame lose focus, so that
  // PressureObserver in the main frame can't receive PressureRecord
  // by default. However, if the main frame is the initiator of active
  // Picture-in-Picture session, PressureObserver in the main frame can
  // receive PressureRecord.
  iframe.contentWindow.focus();

  await new Promise(resolve => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(async () => {
      observer.disconnect();
      iframe.remove();
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture();
      }
      video.remove();
    });
    observer.observe('cpu');
  });
}, 'Observer should receive PressureRecord if associated document is the initiator of active Picture-in-Picture session');

promise_test(async t => {
  await setMediaPermission();
  const stream =
      await navigator.mediaDevices.getUserMedia({video: true, audio: true});
  assert_true(stream.active);

  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  // Focus on the iframe to make the main frame lose focus, so that
  // PressureObserver in the main frame can't receive PressureRecord
  // by default. However, if the main frame's browsing context is capturing,
  // PressureObserver in the main frame can receive PressureRecord.
  iframe.contentWindow.focus();

  await new Promise(resolve => {
    const observer = new PressureObserver(resolve);
    t.add_cleanup(async () => {
      observer.disconnect();
      iframe.remove();
      stream.getTracks().forEach(track => track.stop());
    });
    observer.observe('cpu');
  });
}, 'Observer should receive PressureRecord if browsing context is capturing');

promise_test(async t => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  // Focus on the iframe to make the main frame lose focus, so that
  // PressureObserver in the main frame can't receive PressureRecord
  // by default.
  iframe.contentWindow.focus();

  const observer = new PressureObserver(() => {
    assert_unreached('The observer callback should not be called');
  });
  t.add_cleanup(() => {
    observer.disconnect();
    iframe.remove();
  });

  return new Promise(resolve => t.step_timeout(resolve, 2000));
}, 'Observer should not receive PressureRecord when top-level browsing context does not have system focus');

promise_test(async t => {
  const iframe = document.createElement('iframe');
  document.body.appendChild(iframe);
  // Focus on the main frame to make the iframe lose focus, so that
  // PressureObserver in the iframe can't receive PressureRecord by default.
  // However, if the iframe is same-origin with the main frame and the main
  // frame has focus, PressureObserver in iframe can receive PressureRecord.
  window.focus();

  await new Promise(resolve => {
    const observer = new iframe.contentWindow.PressureObserver(resolve);
    t.add_cleanup(() => {
      observer.disconnect();
      iframe.remove();
    });
    observer.observe('cpu');
  });
}, 'Observer in iframe should receive PressureRecord when focused on same-origin main frame');

promise_test(async t => {
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
  });
}, 'Observer in iframe should not receive PressureRecord when focused on cross-origin main frame');
