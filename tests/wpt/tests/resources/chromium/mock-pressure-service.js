import {PressureStatus} from '/gen/services/device/public/mojom/pressure_manager.mojom.m.js'
import {PressureSource, PressureState} from '/gen/services/device/public/mojom/pressure_update.mojom.m.js'
import {WebPressureManager, WebPressureManagerReceiver} from '/gen/third_party/blink/public/mojom/compute_pressure/web_pressure_manager.mojom.m.js'

class MockWebPressureService {
  constructor() {
    this.receiver_ = new WebPressureManagerReceiver(this);
    this.interceptor_ =
        new MojoInterfaceInterceptor(WebPressureManager.$interfaceName);
    this.interceptor_.oninterfacerequest = e => {
      this.receiver_.$.bindHandle(e.handle);
    };
    this.reset();
    this.mojomSourceType_ = new Map([['cpu', PressureSource.kCpu]]);
    this.mojomStateType_ = new Map([
      ['nominal', PressureState.kNominal], ['fair', PressureState.kFair],
      ['serious', PressureState.kSerious], ['critical', PressureState.kCritical]
    ]);
    this.pressureServiceReadingTimerId_ = null;
  }

  start() {
    this.interceptor_.start();
  }

  stop() {
    this.stopPlatformCollector();
    this.receiver_.$.close();
    this.interceptor_.stop();

    // Wait for an event loop iteration to let any pending mojo commands in
    // the pressure service finish.
    return new Promise(resolve => setTimeout(resolve, 0));
  }

  reset() {
    this.observers_ = [];
    this.pressureUpdate_ = null;
    this.pressureServiceReadingTimerId_ = null;
    this.pressureStatus_ = PressureStatus.kOk;
    this.updatesDelivered_ = 0;
  }

  async addClient(observer, source) {
    if (this.observers_.indexOf(observer) >= 0)
      throw new Error('addClient() has already been called');

    // TODO(crbug.com/1342184): Consider other sources.
    // For now, "cpu" is the only source.
    if (source !== PressureSource.kCpu)
      throw new Error('Call addClient() with a wrong PressureSource');

    observer.onConnectionError.addListener(() => {
      // Remove this observer from observer array.
      this.observers_.splice(this.observers_.indexOf(observer), 1);
    });
    this.observers_.push(observer);

    return {status: this.pressureStatus_};
  }

  startPlatformCollector(sampleInterval) {
    if (sampleInterval === 0)
      return;

    if (this.pressureServiceReadingTimerId_ != null)
      this.stopPlatformCollector();

    this.pressureServiceReadingTimerId_ = self.setInterval(() => {
      if (this.pressureUpdate_ === null || this.observers_.length === 0)
        return;

      // Because we cannot retrieve directly the timeOrigin internal in
      // TimeTicks from Chromium, we need to create a timestamp that
      // would help to test basic functionality.
      // by multiplying performance.timeOrigin by 10 we make sure that the
      // origin is higher than the internal time origin in TimeTicks.
      // performance.now() allows us to have an increase matching the TimeTicks
      // that would be read from the platform collector.
      this.pressureUpdate_.timestamp = {
        internalValue:
            Math.round((performance.timeOrigin * 10) + performance.now()) * 1000
      };
      for (let observer of this.observers_)
        observer.onPressureUpdated(this.pressureUpdate_);
      this.updatesDelivered_++;
    }, sampleInterval);
  }

  stopPlatformCollector() {
    if (this.pressureServiceReadingTimerId_ != null) {
      self.clearInterval(this.pressureServiceReadingTimerId_);
      this.pressureServiceReadingTimerId_ = null;
    }
    this.updatesDelivered_ = 0;
  }

  updatesDelivered() {
    return this.updatesDelivered_;
  }

  setPressureUpdate(source, state) {
    if (!this.mojomSourceType_.has(source))
      throw new Error(`PressureSource '${source}' is invalid`);

    if (!this.mojomStateType_.has(state))
      throw new Error(`PressureState '${state}' is invalid`);

    this.pressureUpdate_ = {
      source: this.mojomSourceType_.get(source),
      state: this.mojomStateType_.get(state),
    };
  }

  setExpectedFailure(expectedException) {
    assert_true(
        expectedException instanceof DOMException,
        'setExpectedFailure() expects a DOMException instance');
    if (expectedException.name === 'NotSupportedError') {
      this.pressureStatus_ = PressureStatus.kNotSupported;
    } else {
      throw new TypeError(
          `Unexpected DOMException '${expectedException.name}'`);
    }
  }
}

export const mockPressureService = new MockWebPressureService();
