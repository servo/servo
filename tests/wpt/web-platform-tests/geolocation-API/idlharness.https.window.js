// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js

// https://www.w3.org/TR/geolocation-API/

window.onload = async () => {
  await test_driver.set_permission({ name: "geolocation" }, "denied");
  const positionError = await new Promise((resolve, reject) => {
    navigator.geolocation.getCurrentPosition(reject, resolve);
  });

  await test_driver.set_permission({ name: "geolocation" }, "granted");
  const position = await new Promise((resolve, reject) => {
    navigator.geolocation.getCurrentPosition(resolve, reject);
  });

  idl_test(["geolocation"], ["hr-time", "html"], (idl_array) => {
    idl_array.add_objects({
      Navigator: ["navigator"],
      Geolocation: ["navigator.geolocation"],
      GeolocationPositionError: [positionError],
      GeolocationPosition: [position],
      GeolocationCoordinates: [position.coords],
    });
  });
};
