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

  const devices = [];
  // Errors will be surfaced in idlharness.js's test_object below.
  try {
    const list = await navigator.mediaDevices.enumerateDevices();
    for (const item of list) {
      switch (item.kind) {
      case 'audioinput':
      case 'videoinput':
      case 'audiooutput':
        self[item.kind] = item;
        devices.push(item.kind);
      default:
        assert_unreached(
          'media.kind should be one of "audioinput", "videoinput", or "audiooutput".');
      }
    }
  } catch (e) {}

  try {
    self.stream = await navigator.mediaDevices.getUserMedia({audio: true});
    self.track = stream.getTracks()[0];
    self.trackEvent = new MediaStreamTrackEvent("type", {
      track: track,
    });
  } catch (e) { throw e}

  idl_array.add_objects({
    InputDeviceInfo: devices,
    MediaStream: ['stream', 'new MediaStream()'],
    Navigator: ['navigator'],
    MediaDevices: ['navigator.mediaDevices'],
    MediaStreamTrack: ['track'],
    MediaStreamTrackEvent: ['trackEvent'],
  });
  idl_array.test();
}, 'mediacapture-streams interfaces.');
