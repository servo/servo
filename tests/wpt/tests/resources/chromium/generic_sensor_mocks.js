import {ReportingMode, Sensor, SensorClientRemote, SensorReceiver, SensorRemote, SensorType} from '/gen/services/device/public/mojom/sensor.mojom.m.js';
import {SensorCreationResult, SensorInitParams_READ_BUFFER_SIZE_FOR_TESTS} from '/gen/services/device/public/mojom/sensor_provider.mojom.m.js';
import {WebSensorProvider, WebSensorProviderReceiver} from '/gen/third_party/blink/public/mojom/sensor/web_sensor_provider.mojom.m.js';

// A "sliding window" that iterates over |data| and returns one item at a
// time, advancing and wrapping around as needed. |data| must be an array of
// arrays.
self.RingBuffer = class {
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
};

class DefaultSensorTraits {
  // https://w3c.github.io/sensors/#threshold-check-algorithm
  static isSignificantlyDifferent(reading1, reading2) {
    return true;
  }

  // https://w3c.github.io/sensors/#reading-quantization-algorithm
  static roundToMultiple(reading) {
    return reading;
  }

  // https://w3c.github.io/ambient-light/#ambient-light-threshold-check-algorithm
  static areReadingsEqual(reading1, reading2) {
    return false;
  }
}

class AmbientLightSensorTraits extends DefaultSensorTraits {
  // https://w3c.github.io/ambient-light/#reduce-sensor-accuracy
  static #ROUNDING_MULTIPLE = 50;
  static #SIGNIFICANCE_THRESHOLD = 25;

  // https://w3c.github.io/ambient-light/#ambient-light-threshold-check-algorithm
  static isSignificantlyDifferent([illuminance1], [illuminance2]) {
    return Math.abs(illuminance1 - illuminance2) >=
        this.#SIGNIFICANCE_THRESHOLD;
  }

  // https://w3c.github.io/ambient-light/#ambient-light-reading-quantization-algorithm
  static roundToMultiple(reading) {
    const illuminance = reading[0];
    const scaledValue =
        illuminance / AmbientLightSensorTraits.#ROUNDING_MULTIPLE;
    let roundedReading = reading.splice();

    if (illuminance < 0.0) {
      roundedReading[0] = -AmbientLightSensorTraits.#ROUNDING_MULTIPLE *
          Math.floor(-scaledValue + 0.5);
    } else {
      roundedReading[0] = AmbientLightSensorTraits.#ROUNDING_MULTIPLE *
          Math.floor(scaledValue + 0.5);
    }

    return roundedReading;
  }

  // https://w3c.github.io/ambient-light/#ambient-light-threshold-check-algorithm
  static areReadingsEqual([illuminance1], [illuminance2]) {
    return illuminance1 === illuminance2;
  }
}

self.GenericSensorTest = (() => {
  // Default sensor frequency in default configurations.
  const DEFAULT_FREQUENCY = 5;

  // Class that mocks Sensor interface defined in
  // https://cs.chromium.org/chromium/src/services/device/public/mojom/sensor.mojom
  class MockSensor {
    static #BUFFER_OFFSET_TIMESTAMP = 1;
    static #BUFFER_OFFSET_READINGS = 2;

    constructor(sensorRequest, buffer, reportingMode, sensorType) {
      this.client_ = null;
      this.startShouldFail_ = false;
      this.notifyOnReadingChange_ = true;
      this.reportingMode_ = reportingMode;
      this.sensorType_ = sensorType;
      this.sensorReadingTimerId_ = null;
      this.readingData_ = null;
      this.requestedFrequencies_ = [];
      // The Blink implementation (third_party/blink/renderer/modules/sensor/sensor.cc)
      // sets a timestamp by creating a DOMHighResTimeStamp from a given platform timestamp.
      // In this mock implementation we use a starting value
      // and an increment step value that resemble a platform timestamp reasonably enough.
      this.timestamp_ = window.performance.timeOrigin;
      // |buffer| represents a SensorReadingSharedBuffer on the C++ side in
      // Chromium. It consists, in this order, of a
      // SensorReadingField<OneWriterSeqLock> (an 8-byte union that includes
      // 32-bit integer used by the lock class), and a SensorReading consisting
      // of an 8-byte timestamp and 4 8-byte reading fields.
      //
      // |this.buffer_[0]| is zeroed by default, which allows OneWriterSeqLock
      // to work with our custom memory buffer that did not actually create a
      // OneWriterSeqLock instance. It is never changed manually here.
      //
      // Use MockSensor.#BUFFER_OFFSET_TIMESTAMP and
      // MockSensor.#BUFFER_OFFSET_READINGS to access the other positions in
      // |this.buffer_| without having to hardcode magic numbers in the code.
      this.buffer_ = buffer;
      this.buffer_.fill(0);
      this.receiver_ = new SensorReceiver(this);
      this.receiver_.$.bindHandle(sensorRequest.handle);
      this.lastRawReading_ = null;
      this.lastRoundedReading_ = null;

      if (sensorType == SensorType.AMBIENT_LIGHT) {
        this.sensorTraits = AmbientLightSensorTraits;
      } else {
        this.sensorTraits = DefaultSensorTraits;
      }
    }

    // Returns default configuration.
    async getDefaultConfiguration() {
      return { frequency: DEFAULT_FREQUENCY };
    }

    // Adds configuration for the sensor and starts reporting fake data
    // through setSensorReading function.
    async addConfiguration(configuration) {
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

    resume() {
      this.startReading();
    }

    suspend() {
      this.stopReading();
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
      this.receiver_.$.close();
      this.lastRawReading_ = null;
      this.lastRoundedReading_ = null;
    }

    // Sets fake data that is used to deliver sensor reading updates.
    setSensorReading(readingData) {
      this.readingData_ = new RingBuffer(readingData);
    }

    // This is a workaround to accommodate Blink's Device Orientation
    // implementation. In general, all tests should use setSensorReading()
    // instead.
    setSensorReadingImmediately(readingData) {
      this.setSensorReading(readingData);

      const reading = this.readingData_.value();
      this.buffer_.set(reading, MockSensor.#BUFFER_OFFSET_READINGS);
      this.buffer_[MockSensor.#BUFFER_OFFSET_TIMESTAMP] = this.timestamp_++;
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
          if (!Array.isArray(reading)) {
            throw new TypeError("startReading(): The readings passed to " +
              "setSensorReading() must be arrays");
          }

          if (this.reportingMode_ == ReportingMode.ON_CHANGE &&
              this.lastRawReading_ !== null &&
              !this.sensorTraits.isSignificantlyDifferent(
                  this.lastRawReading_, reading)) {
            // In case new value is not significantly different compared to
            // old value, new value is not sent.
            return;
          }

          this.lastRawReading_ = reading.slice();
          const roundedReading = this.sensorTraits.roundToMultiple(reading);

          if (this.reportingMode_ == ReportingMode.ON_CHANGE &&
              this.lastRoundedReading_ !== null &&
              this.sensorTraits.areReadingsEqual(
                roundedReading, this.lastRoundedReading_)) {
            // In case new rounded value is not different compared to old
            // value, new value is not sent.
            return;
          }
          this.buffer_.set(roundedReading, MockSensor.#BUFFER_OFFSET_READINGS);
          this.lastRoundedReading_ = roundedReading;
        }

        // For all tests sensor reading should have monotonically
        // increasing timestamp.
        this.buffer_[MockSensor.#BUFFER_OFFSET_TIMESTAMP] = this.timestamp_++;

        if (this.reportingMode_ === ReportingMode.ON_CHANGE &&
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
      this.buffer_.fill(0);
      this.lastRawReading_ = null;
      this.lastRoundedReading_ = null;
    }

    getSamplingFrequency() {
      if (this.requestedFrequencies_.length == 0) {
        throw new Error("getSamplingFrequency(): No configured frequency");
      }
       return this.requestedFrequencies_[0];
    }

    isReadingData() {
      return this.sensorReadingTimerId_ != null;
    }
  }

  // Class that mocks the WebSensorProvider interface defined in
  // https://cs.chromium.org/chromium/src/third_party/blink/public/mojom/sensor/web_sensor_provider.mojom
  class MockSensorProvider {
    constructor() {
      this.readingSizeInBytes_ =
          Number(SensorInitParams_READ_BUFFER_SIZE_FOR_TESTS);
      this.sharedBufferSizeInBytes_ =
          this.readingSizeInBytes_ * (SensorType.MAX_VALUE + 1);
      let rv = Mojo.createSharedBuffer(this.sharedBufferSizeInBytes_);
      if (rv.result != Mojo.RESULT_OK) {
        throw new Error('MockSensorProvider: Failed to create shared buffer');
      }
      const handle = rv.handle;
      rv = handle.mapBuffer(0, this.sharedBufferSizeInBytes_);
      if (rv.result != Mojo.RESULT_OK) {
        throw new Error("MockSensorProvider: Failed to map shared buffer");
      }
      this.shmemArrayBuffer_ = rv.buffer;
      rv = handle.duplicateBufferHandle({readOnly: true});
      if (rv.result != Mojo.RESULT_OK) {
        throw new Error(
            'MockSensorProvider: failed to duplicate shared buffer');
      }
      this.readOnlySharedBufferHandle_ = rv.handle;
      this.activeSensors_ = new Map();
      this.resolveFuncs_ = new Map();
      this.getSensorShouldFail_ = new Map();
      this.permissionsDenied_ = new Map();
      this.maxFrequency_ = 60;
      this.minFrequency_ = 1;
      this.mojomSensorType_ = new Map([
        ['Accelerometer', SensorType.ACCELEROMETER],
        ['LinearAccelerationSensor', SensorType.LINEAR_ACCELERATION],
        ['GravitySensor', SensorType.GRAVITY],
        ['AmbientLightSensor', SensorType.AMBIENT_LIGHT],
        ['Gyroscope', SensorType.GYROSCOPE],
        ['Magnetometer', SensorType.MAGNETOMETER],
        ['AbsoluteOrientationSensor',
            SensorType.ABSOLUTE_ORIENTATION_QUATERNION],
        ['AbsoluteOrientationEulerAngles',
            SensorType.ABSOLUTE_ORIENTATION_EULER_ANGLES],
        ['RelativeOrientationSensor',
            SensorType.RELATIVE_ORIENTATION_QUATERNION],
        ['RelativeOrientationEulerAngles',
            SensorType.RELATIVE_ORIENTATION_EULER_ANGLES],
        ['ProximitySensor', SensorType.PROXIMITY]
      ]);
      this.receiver_ = new WebSensorProviderReceiver(this);

      this.interceptor_ =
        new MojoInterfaceInterceptor(WebSensorProvider.$interfaceName);
      this.interceptor_.oninterfacerequest = e => {
        this.bindToPipe(e.handle);
      };
      this.interceptor_.start();
    }

    // Returns initialized Sensor proxy to the client.
    async getSensor(type) {
      if (this.getSensorShouldFail_.get(type)) {
        return {result: SensorCreationResult.ERROR_NOT_AVAILABLE,
                initParams: null};
      }
      if (this.permissionsDenied_.get(type)) {
        return {result: SensorCreationResult.ERROR_NOT_ALLOWED,
                initParams: null};
      }

      const offset = type * this.readingSizeInBytes_;
      const reportingMode = ReportingMode.ON_CHANGE;

      const sensor = new SensorRemote();
      if (!this.activeSensors_.has(type)) {
        const shmemView = new Float64Array(
            this.shmemArrayBuffer_, offset,
            this.readingSizeInBytes_ / Float64Array.BYTES_PER_ELEMENT);
        const mockSensor = new MockSensor(
            sensor.$.bindNewPipeAndPassReceiver(), shmemView, reportingMode,
            type);
        this.activeSensors_.set(type, mockSensor);
        this.activeSensors_.get(type).client_ = new SensorClientRemote();
      }

      const rv = this.readOnlySharedBufferHandle_.duplicateBufferHandle(
          {readOnly: true});
      if (rv.result != Mojo.RESULT_OK) {
        throw new Error('getSensor(): failed to duplicate shared buffer');
      }

      const defaultConfig = { frequency: DEFAULT_FREQUENCY };
      // Consider sensor traits to meet assertions in C++ code (see
      // services/device/public/cpp/generic_sensor/sensor_traits.h)
      if (type == SensorType.AMBIENT_LIGHT || type == SensorType.MAGNETOMETER) {
        this.maxFrequency_ = Math.min(10, this.maxFrequency_);
      }

      const client = this.activeSensors_.get(type).client_;
      const initParams = {
        sensor,
        clientReceiver: client.$.bindNewPipeAndPassReceiver(),
        memory: {buffer: rv.handle},
        bufferOffset: BigInt(offset),
        mode: reportingMode,
        defaultConfiguration: defaultConfig,
        minimumFrequency: this.minFrequency_,
        maximumFrequency: this.maxFrequency_
      };

      if (this.resolveFuncs_.has(type)) {
        for (let resolveFunc of this.resolveFuncs_.get(type)) {
          resolveFunc(this.activeSensors_.get(type));
        }
        this.resolveFuncs_.delete(type);
      }

      return {result: SensorCreationResult.SUCCESS, initParams};
    }

    // Binds object to mojo message pipe
    bindToPipe(pipe) {
      this.receiver_.$.bindHandle(pipe);
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
      this.receiver_.$.close();
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
      if (typeof type != "number") {
        throw new TypeError(`getCreatedSensor(): Invalid sensor type ${sensorType}`);
      }

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
      // testdriver.js only works in the top-level browsing context, so do
      // nothing if we're in e.g. an iframe.
      if (window.parent === window) {
        for (const entry
                 of ['accelerometer', 'gyroscope', 'magnetometer',
                     'ambient-light-sensor']) {
          await test_driver.set_permission({name: entry}, 'granted');
        }
      }

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
