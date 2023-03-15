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
  iframe.src = get_host_info().HTTPS_REMOTE_ORIGIN +
      '/compute-pressure/resources/support-iframe.html';
  const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
  document.body.appendChild(iframe);
  await iframeLoadWatcher.wait_for('load');
  // Focus on the cross-origin iframe, so that PressureObserver in the main
  // frame can't receive PressureRecord by default. However, if the main
  // frame is the initiator of active Picture-in-Picture session,
  // PressureObserver in the main frame can receive PressureRecord.
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
  iframe.src = get_host_info().HTTPS_REMOTE_ORIGIN +
      '/compute-pressure/resources/support-iframe.html';
  const iframeLoadWatcher = new EventWatcher(t, iframe, 'load');
  document.body.appendChild(iframe);
  await iframeLoadWatcher.wait_for('load');
  // Focus on the cross-origin iframe, so that PressureObserver in the main
  // frame can't receive PressureRecord by default. However, if the main
  // frame's browsing context is capturing, PressureObserver in the main
  // frame can receive PressureRecord.
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
