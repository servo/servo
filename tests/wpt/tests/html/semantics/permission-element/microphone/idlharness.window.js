// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['microphone-element.tentative'],
  ['html', 'dom', 'mediacapture-streams'],
  (idl_array) => {
    idl_array.add_objects({
      HTMLMicrophoneElement: ["document.createElement('microphone')"],
    });
  });

