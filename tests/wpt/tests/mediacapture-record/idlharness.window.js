// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

// https://w3c.github.io/mediacapture-record/

idl_test(
  ['mediastream-recording'],
  ['mediacapture-streams', 'FileAPI', 'html', 'dom', 'webidl'],
  idl_array => {
    // Ignored errors will be surfaced in idlharness.js's test_object below.
    let recorder, blob, error;
    try {
      const canvas = document.createElement('canvas');
      document.body.appendChild(canvas);
      const context = canvas.getContext("2d");
      context.fillStyle = "red";
      context.fillRect(0, 0, 10, 10);
      const stream = canvas.captureStream();
      recorder = new MediaRecorder(stream);
    } catch(e) {}
    idl_array.add_objects({ MediaRecorder: [recorder] });

    try {
      blob = new BlobEvent("type", {
        data: new Blob(),
        timecode: performance.now(),
      });
    } catch(e) {}
    idl_array.add_objects({ BlobEvent: [blob] });

    try {
      error = new MediaRecorderErrorEvent("type", {
        error: new DOMException,
      });
    } catch(e) {}
    idl_array.add_objects({ MediaRecorderErrorEvent: [error] });
  }
);
