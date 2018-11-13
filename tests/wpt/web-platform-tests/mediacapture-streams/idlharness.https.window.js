// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/mediacapture-main/


promise_test(async () => {
  const srcs = ['mediacapture-streams','dom','html'];
  const [idl, dom, html] = await Promise.all(
    srcs.map(i => fetch(`/interfaces/${i}.idl`).then(r => r.text())));

  const idl_array = new IdlArray();
  idl_array.add_idls(idl);
  idl_array.add_dependency_idls(html);
  idl_array.add_dependency_idls(dom);

  let input, media;
  // Errors will be surfaced in idlharness.js's test_object below.
  try {
    const list = await navigator.mediaDevices.enumerateDevices();
    for (const item of list) {
      switch (item.kind) {
      case 'audioinput':
      case 'videoinput':
        input = item;
      case 'audiooutput':
        media = item;
      default:
        assert_unreached(
          'media.kind should be one of "audioinput", "videoinput", or "audiooutput".');
      }
    }
  } catch (e) {}

  let track, trackEvent;
  try {
    const stream = await navigator.mediaDevices.getUserMedia({audio: true});
    track = stream.getTracks()[0];
    trackEvent = new MediaStreamTrackEvent("type", {
      track: track,
    });
  } catch (e) { throw e}

  if (input) {
    idl_array.add_objects({ InputDeviceInfo: [input] });
  } else {
    idl_array.add_objects({ MediaDeviceInfo: [media] });
  }
  idl_array.add_objects({
    MediaStream: ['new MediaStream()'],
    Navigator: ['navigator'],
    MediaDevices: ['navigator.mediaDevices'],
    MediaStreamTrack: [track],
    MediaStreamTrackEvent: [trackEvent],
    OverconstrainedErrorEvent: ['new OverconstrainedErrorEvent("type", {})'],
  });
  idl_array.test();
}, 'mediacapture-streams interfaces.');
