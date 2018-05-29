The `generic-sensor-tests.js` tests require an implementation of
the `GenericSensorTest` interface, which should emulate platform
sensor backends. The `GenericSensorTest` interface is defined as:

```
  class GenericSensorTest {
    async initialize();  // Sets up the testing enviroment.
    async reset(); // Frees the resources.
  };
```

The Chromium implementation of the `GenericSensorTest` interface is located in
[generic_sensor_mocks.js](../resources/chromium/generic_sensor_mocks.js).

Other browser vendors should provide their own implementations of
the `GenericSensorTest` interface.

[Known issue](https://github.com/web-platform-tests/wpt/issues/9686): a
WebDriver extension is a better approach for the Generic Sensor tests
automation.
