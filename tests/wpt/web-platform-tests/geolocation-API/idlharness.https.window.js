// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://www.w3.org/TR/geolocation-API/

idl_test(
  ['geolocation-API'],
  ['html'],
  idl_array => {
    self.audio = document.createElement('audio');
    self.video = document.createElement('video');
    idl_array.add_objects({
      Navigator: ['navigator'],
      Geolocation: ['navigator.geolocation'],
    });
  }
);
