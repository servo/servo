The `resources/orientation-event-helpers.js` tests depend on the implementation of
the `GenericSensorTest` interface which is defined in [README.md](../generic-sensor/README.md).

The Chromium implementation of the `GenericSensorTest` interface is located in
[generic_sensor_mocks.js](../resources/chromium/generic_sensor_mocks.js).

Other browser vendors should provide their own implementations of
the `GenericSensorTest` interface.
