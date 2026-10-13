// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['usermedia-element.tentative', 'geolocation-element.tentative'],
  ['html', 'dom', 'permissions', 'geolocation', 'mediacapture-streams'],
  (idl_array) => {
    idl_array.add_objects({
      HTMLUserMediaElement: ["document.createElement('usermedia')"],
    });
  });
