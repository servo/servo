The `resources/generic-sensor-helpers.js` tests require an implementation of
the `GenericSensorTest` interface, which should emulate platform
sensor backends. The `GenericSensorTest` interface is defined as:

```
  class MockSensor {
    // Sets fake data that is used to deliver sensor reading updates.
    async setSensorReading(FrozenArray<double> readingData);
    setStartShouldFail(boolean shouldFail); // Sets flag that forces sensor to fail.
    getSamplingFrequency(); // Return the sampling frequency.
  };

  class MockSensorProvider {
    // Sets flag that forces mock SensorProvider to fail when getSensor() is
    // invoked.
    setGetSensorShouldFail(DOMString sensorType, boolean shouldFail);
    // Sets flag that forces mock SensorProvider to permissions denied when
    // getSensor() is invoked.
    setPermissionsDenied(DOMString sensorType, boolean permissionsDenied);
    getCreatedSensor(DOMString sensorType); // Return `MockSensor` interface.
    setMaximumSupportedFrequency(double frequency); // Sets the maximum frequency.
    setMinimumSupportedFrequency(double frequency); // Sets the minimum frequency.
  }

  class GenericSensorTest {
    initialize();  // Sets up the testing environment.
    async reset(); // Frees the resources.
    getSensorProvider(); // Returns `MockSensorProvider` interface.
  };
```

The Chromium implementation of the `GenericSensorTest` interface is located in
[generic_sensor_mocks.js](../resources/chromium/generic_sensor_mocks.js).

Other browser vendors should provide their own implementations of
the `GenericSensorTest` interface.

[Known issue](https://github.com/web-platform-tests/wpt/issues/9686): a
WebDriver extension is a better approach for the Generic Sensor tests
automation.
