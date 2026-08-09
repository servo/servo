// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['camera-element.tentative'],
  ['html', 'dom', 'mediacapture-streams'],
  (idl_array) => {
    idl_array.add_objects({
      HTMLCameraElement: ["document.createElement('camera')"],
    });
  });

