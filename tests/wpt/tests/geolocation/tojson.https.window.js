// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
'use strict';

function check_coords(original, json, prefix) {
  for (const key of [
    'accuracy',
    'latitude',
    'longitude',
    'altitude',
    'altitudeAccuracy',
    'heading',
    'speed',
    'coords',
    'timestamp',
  ]) {
    assert_equals(original[key], json[key], `${prefix} ${key} entry does not match its toJSON value`);
  }
}

promise_setup(async () => {
  await test_driver.set_permission({ name: "geolocation" }, "granted");

  if (document.readyState != 'complete') {
    await new Promise(resolve => {
      window.addEventListener('load', resolve, {once: true});
    });
  }
}, 'Grant permission and wait for the document to be fully active.');

promise_test(async (t) => {
  const position = await new Promise((resolve, reject) => {
    navigator.geolocation.getCurrentPosition(
      t.step_func((position) => {
        resolve(position);
      }),
      t.step_func((error) => {
        reject(error.message);
      }),
    );
  });

  assert_equals(typeof(position.toJSON), 'function');

  const json = position.toJSON();
  assert_equals(position.timestamp, json.timestamp, 'GeolocationPosition timestamp entry does not match its toJSON value');
  check_coords(position.coords, json.coords, 'GeolocationPosition coords');

  assert_equals(typeof(position.coords.toJSON), 'function');
  check_coords(position.coords, position.coords.toJSON(), 'GeolocationCoordinates');
}, 'Test toJSON() in GeolocationPosition and GeolocationCoordinates.');
