'use strict';

// A "sliding window" that iterates over |data| and returns one item at a
// time, advancing and wrapping around as needed. |data| must be an array of
// arrays.
class RingBuffer {
  constructor(data) {
    this.bufferPosition_ = 0;
    // Validate |data|'s format and deep-copy every element.
    this.data_ = Array.from(data, element => {
      if (!Array.isArray(element)) {
        throw new TypeError('Every |data| element must be an array.');
      }
      return Array.from(element);
    })
  }

  next() {
    const value = this.data_[this.bufferPosition_];
    this.bufferPosition_ = (this.bufferPosition_ + 1) % this.data_.length;
    return { done: false, value: value };
  }

  value() {
    return this.data_[this.bufferPosition_];
  }

  [Symbol.iterator]() {
    return this;
  }
}

var GenericSensorTest = (() => {
  // Default sensor frequency in default configurations.
  const DEFAULT_FREQUENCY = 5;

  // Class that mocks Sensor interface defined in
  // https://cs.chromium.org/chromium/src/services/device/public/mojom/sensor.mojom
  class MockSensor {
    constructor(sensorRequest, handle, offset, size, reportingMode) {
      this.client_ = null;
      this.startShouldFail_ = false;
      this.notifyOnReadingChange_ = true;
      this.reportingMode_ = reportingMode;
      this.sensorReadingTimerId_ = null;
      this.readingData_ = null;
      this.requestedFrequencies_ = [];
      let rv = handle.mapBuffer(offset, size);
      assert_equals(rv.result, Mojo.RESULT_OK, "Failed to map shared buffer");
      this.buffer_ = new Float64Array(rv.buffer);
      this.buffer_.fill(0);
      this.binding_ = new mojo.Binding(device.mojom.Sensor, this,
                                       sensorRequest);
      this.binding_.setConnectionErrorHandler(() => {
        this.reset();
      });
    }

    // Returns default configuration.
    async getDefaultConfiguration() {
      return { frequency: DEFAULT_FREQUENCY };
    }

    // Adds configuration for the sensor and starts reporting fake data
    // through setSensorReading function.
    async addConfiguration(configuration) {
      assert_not_equals(configuration, null, "Invalid sensor configuration.");

      this.requestedFrequencies_.push(configuration.frequency);
      // Sort using descending order.
      this.requestedFrequencies_.sort(
          (first, second) => { return second - first });

      if (!this.startShouldFail_ )
        this.startReading();

      return { success: !this.startShouldFail_ };
    }

    // Removes sensor configuration from the list of active configurations and
    // stops notification about sensor reading changes if
    // requestedFrequencies_ is empty.
    removeConfiguration(configuration) {
      const index = this.requestedFrequencies_.indexOf(configuration.frequency);
      if (index == -1)
        return;

      this.requestedFrequencies_.splice(index, 1);
      if (this.requestedFrequencies_.length === 0)
        this.stopReading();
    }

    // ConfigureReadingChangeNotifications(bool enabled)
    // Configures whether to report a reading change when in ON_CHANGE
    // reporting mode.
    configureReadingChangeNotifications(notifyOnReadingChange) {
      this.notifyOnReadingChange_ = notifyOnReadingChange;
    }

    // Mock functions

    // Resets mock Sensor state.
    reset() {
      this.stopReading();
      this.startShouldFail_ = false;
      this.requestedFrequencies_ = [];
      this.notifyOnReadingChange_ = true;
      this.readingData_ = null;
      this.buffer_.fill(0);
      this.binding_.close();
    }

    // Sets fake data that is used to deliver sensor reading updates.
    async setSensorReading(readingData) {
      this.readingData_ = new RingBuffer(readingData);
      return this;
    }

    // This is a workaround to accommodate Blink's Device Orientation
    // implementation. In general, all tests should use setSensorReading()
    // instead.
    setSensorReadingImmediately(readingData) {
      this.setSensorReading(readingData);

      const reading = this.readingData_.value();
      this.buffer_.set(reading, 2);
      this.buffer_[1] = window.performance.now() * 0.001;
    }

    // Sets flag that forces sensor to fail when addConfiguration is invoked.
    setStartShouldFail(shouldFail) {
      this.startShouldFail_ = shouldFail;
    }

    startReading() {
      if (this.readingData_ != null) {
        this.stopReading();
      }
      let maxFrequencyUsed = this.requestedFrequencies_[0];
      let timeout = (1 / maxFrequencyUsed) * 1000;
      this.sensorReadingTimerId_ = window.setInterval(() => {
        if (this.readingData_) {
          // |buffer_| is a TypedArray, so we need to make sure pass an
          // array to set().
          const reading = this.readingData_.next().value;
          assert_true(Array.isArray(reading), "The readings passed to " +
              "setSensorReading() must be arrays.");
          this.buffer_.set(reading, 2);
        }
        // For all tests sensor reading should have monotonically
        // increasing timestamp in seconds.
        this.buffer_[1] = window.performance.now() * 0.001;
        if (this.reportingMode_ === device.mojom.ReportingMode.ON_CHANGE &&
            this.notifyOnReadingChange_) {
          this.client_.sensorReadingChanged();
        }
      }, timeout);
    }

    stopReading() {
      if (this.sensorReadingTimerId_ != null) {
        window.clearInterval(this.sensorReadingTimerId_);
        this.sensorReadingTimerId_ = null;
      }
    }

    getSamplingFrequency() {
       assert_true(this.requestedFrequencies_.length > 0);
       return this.requestedFrequencies_[0];
    }
  }

  // Class that mocks SensorProvider interface defined in
  // https://cs.chromium.org/chromium/src/services/device/public/mojom/sensor_provider.mojom
  class MockSensorProvider {
    constructor() {
      this.readingSizeInBytes_ =
          device.mojom.SensorInitParams.READ_BUFFER_SIZE_FOR_TESTS;
      this.sharedBufferSizeInBytes_ = this.readingSizeInBytes_ *
              (device.mojom.SensorType.MAX_VALUE + 1);
      const rv = Mojo.createSharedBuffer(this.sharedBufferSizeInBytes_);
      assert_equals(rv.result, Mojo.RESULT_OK, "Failed to create buffer");
      this.sharedBufferHandle_ = rv.handle;
      this.activeSensors_ = new Map();
      this.resolveFuncs_ = new Map();
      this.getSensorShouldFail_ = new Map();
      this.permissionsDenied_ = new Map();
      this.maxFrequency_ = 60;
      this.minFrequency_ = 1;
      this.mojomSensorType_ = new Map([
        ['Accelerometer', device.mojom.SensorType.ACCELEROMETER],
        ['LinearAccelerationSensor',
            device.mojom.SensorType.LINEAR_ACCELERATION],
        ['AmbientLightSensor', device.mojom.SensorType.AMBIENT_LIGHT],
        ['Gyroscope', device.mojom.SensorType.GYROSCOPE],
        ['Magnetometer', device.mojom.SensorType.MAGNETOMETER],
        ['AbsoluteOrientationSensor',
            device.mojom.SensorType.ABSOLUTE_ORIENTATION_QUATERNION],
        ['AbsoluteOrientationEulerAngles',
            device.mojom.SensorType.ABSOLUTE_ORIENTATION_EULER_ANGLES],
        ['RelativeOrientationSensor',
            device.mojom.SensorType.RELATIVE_ORIENTATION_QUATERNION],
        ['RelativeOrientationEulerAngles',
            device.mojom.SensorType.RELATIVE_ORIENTATION_EULER_ANGLES],
        ['ProximitySensor', device.mojom.SensorType.PROXIMITY]
      ]);
      this.binding_ = new mojo.Binding(device.mojom.SensorProvider, this);

      this.interceptor_ =
          new MojoInterfaceInterceptor(device.mojom.SensorProvider.name);
      this.interceptor_.oninterfacerequest = e => {
        this.bindToPipe(e.handle);
      };
      this.interceptor_.start();
    }

    // Returns initialized Sensor proxy to the client.
    async getSensor(type) {
      if (this.getSensorShouldFail_.get(type)) {
        return {result: device.mojom.SensorCreationResult.ERROR_NOT_AVAILABLE,
                initParams: null};
      }
      if (this.permissionsDenied_.get(type)) {
        return {result: device.mojom.SensorCreationResult.ERROR_NOT_ALLOWED,
                initParams: null};
      }

      const offset = type * this.readingSizeInBytes_;
      const reportingMode = device.mojom.ReportingMode.ON_CHANGE;

      const sensorPtr = new device.mojom.SensorPtr();
      if (!this.activeSensors_.has(type)) {
        const mockSensor = new MockSensor(
            mojo.makeRequest(sensorPtr), this.sharedBufferHandle_, offset,
            this.readingSizeInBytes_, reportingMode);
        this.activeSensors_.set(type, mockSensor);
        this.activeSensors_.get(type).client_ =
            new device.mojom.SensorClientPtr();
      }

      const rv = this.sharedBufferHandle_.duplicateBufferHandle();

      assert_equals(rv.result, Mojo.RESULT_OK);

      const defaultConfig = { frequency: DEFAULT_FREQUENCY };
      // Consider sensor traits to meet assertions in C++ code (see
      // services/device/public/cpp/generic_sensor/sensor_traits.h)
      if (type == device.mojom.SensorType.AMBIENT_LIGHT ||
          type == device.mojom.SensorType.MAGNETOMETER) {
        this.maxFrequency_ = Math.min(10, this.maxFrequency_);
      }

      // Chromium applies some rounding and other privacy-related measures that
      // can cause ALS not to report a reading when it has not changed beyond a
      // certain threshold compared to the previous illuminance value. Make
      // each reading return a different value that is significantly different
      // from the previous one when setSensorReading() is not called by client
      // code (e.g. run_generic_sensor_iframe_tests()).
      if (type == device.mojom.SensorType.AMBIENT_LIGHT) {
        this.activeSensors_.get(type).setSensorReading([
          [window.performance.now() * 100],
          [(window.performance.now() + 50) * 100]
        ]);
      }

      const initParams = new device.mojom.SensorInitParams({
        sensor: sensorPtr,
        clientReceiver: mojo.makeRequest(this.activeSensors_.get(type).client_),
        memory: rv.handle,
        bufferOffset: offset,
        mode: reportingMode,
        defaultConfiguration: defaultConfig,
        minimumFrequency: this.minFrequency_,
        maximumFrequency: this.maxFrequency_
      });

      if (this.resolveFuncs_.has(type)) {
        for (let resolveFunc of this.resolveFuncs_.get(type)) {
          resolveFunc(this.activeSensors_.get(type));
        }
        this.resolveFuncs_.delete(type);
      }

      return {result: device.mojom.SensorCreationResult.SUCCESS,
              initParams: initParams};
    }

    // Binds object to mojo message pipe
    bindToPipe(pipe) {
      this.binding_.bind(pipe);
      this.binding_.setConnectionErrorHandler(() => {
        this.reset();
      });
    }

    // Mock functions

    // Resets state of mock SensorProvider between test runs.
    reset() {
      for (const sensor of this.activeSensors_.values()) {
        sensor.reset();
      }
      this.activeSensors_.clear();
      this.resolveFuncs_.clear();
      this.getSensorShouldFail_.clear();
      this.permissionsDenied_.clear();
      this.maxFrequency_ = 60;
      this.minFrequency_ = 1;
      this.binding_.close();
      this.interceptor_.stop();
    }

    // Sets flag that forces mock SensorProvider to fail when getSensor() is
    // invoked.
    setGetSensorShouldFail(sensorType, shouldFail) {
      this.getSensorShouldFail_.set(this.mojomSensorType_.get(sensorType),
          shouldFail);
    }

    setPermissionsDenied(sensorType, permissionsDenied) {
      this.permissionsDenied_.set(this.mojomSensorType_.get(sensorType),
          permissionsDenied);
    }

    // Returns mock sensor that was created in getSensor to the layout test.
    getCreatedSensor(sensorType) {
      const type = this.mojomSensorType_.get(sensorType);
      assert_equals(typeof type, "number", "A sensor type must be specified.");

      if (this.activeSensors_.has(type)) {
        return Promise.resolve(this.activeSensors_.get(type));
      }

      return new Promise(resolve => {
        if (!this.resolveFuncs_.has(type)) {
          this.resolveFuncs_.set(type, []);
        }
        this.resolveFuncs_.get(type).push(resolve);
      });
    }

    // Sets the maximum frequency for a concrete sensor.
    setMaximumSupportedFrequency(frequency) {
      this.maxFrequency_ = frequency;
    }

    // Sets the minimum frequency for a concrete sensor.
    setMinimumSupportedFrequency(frequency) {
      this.minFrequency_ = frequency;
    }
  }

  let testInternal = {
    initialized: false,
    sensorProvider: null
  }

  class GenericSensorTestChromium {
    constructor() {
      Object.freeze(this); // Make it immutable.
    }

    async initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      // Grant sensor permissions for Chromium testdriver.
      for (const entry of ['accelerometer', 'gyroscope',
          'magnetometer', 'ambient-light-sensor']) {
        await test_driver.set_permission({ name: entry }, 'granted', false);
      };

      testInternal.sensorProvider = new MockSensorProvider;
      testInternal.initialized = true;
    }
    // Resets state of sensor mocks between test runs.
    async reset() {
      if (!testInternal.initialized)
        throw new Error('Call initialize() before reset().');
      testInternal.sensorProvider.reset();
      testInternal.sensorProvider = null;
      testInternal.initialized = false;

      // Wait for an event loop iteration to let any pending mojo commands in
      // the sensor provider finish.
      await new Promise(resolve => setTimeout(resolve, 0));
    }

    getSensorProvider() {
      return testInternal.sensorProvider;
    }
  }

  return GenericSensorTestChromium;
})();
