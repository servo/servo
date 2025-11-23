// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

idl_test(
  ['geolocation-element.tentative'],
  ['html', 'dom', 'permissions', 'geolocation'],
  (idl_array) => {
    idl_array.add_objects({
      HTMLGeolocationElement: ["document.createElement('geolocation')"],
    });
  });

