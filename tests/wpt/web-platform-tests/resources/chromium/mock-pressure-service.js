import {PressureManager, PressureManagerReceiver, PressureStatus} from '/gen/services/device/public/mojom/pressure_manager.mojom.m.js'
import {PressureSource, PressureState} from '/gen/services/device/public/mojom/pressure_update.mojom.m.js'

class MockPressureService {
  constructor() {
    this.receiver_ = new PressureManagerReceiver(this);
    this.interceptor_ =
        new MojoInterfaceInterceptor(PressureManager.$interfaceName);
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

  startPlatformCollector(sampleRate) {
    if (sampleRate === 0)
      return;

    if (this.pressureServiceReadingTimerId_ != null)
      this.stopPlatformCollector();

    // The following code for calculating the timestamp was taken from
    // https://source.chromium.org/chromium/chromium/src/+/main:third_party/
    // blink/web_tests/http/tests/resources/
    // geolocation-mock.js;l=131;drc=37a9b6c03b9bda9fcd62fc0e5e8016c278abd31f

    // The new Date().getTime() returns the number of milliseconds since the
    // UNIX epoch (1970-01-01 00::00:00 UTC), while |internalValue| of the
    // device.mojom.PressureUpdate represents the value of microseconds since
    // the Windows FILETIME epoch (1601-01-01 00:00:00 UTC). So add the delta
    // when sets the |internalValue|. See more info in //base/time/time.h.
    const windowsEpoch = Date.UTC(1601, 0, 1, 0, 0, 0, 0);
    const unixEpoch = Date.UTC(1970, 0, 1, 0, 0, 0, 0);
    // |epochDeltaInMs| equals to base::Time::kTimeTToMicrosecondsOffset.
    const epochDeltaInMs = unixEpoch - windowsEpoch;

    const timeout = (1 / sampleRate) * 1000;
    this.pressureServiceReadingTimerId_ = self.setInterval(() => {
      if (this.pressureUpdate_ === null || this.observers_.length === 0)
        return;
      this.pressureUpdate_.timestamp = {
        internalValue: BigInt((new Date().getTime() + epochDeltaInMs) * 1000)
      };
      for (let observer of this.observers_)
        observer.onPressureUpdated(this.pressureUpdate_);
      this.updatesDelivered_++;
    }, timeout);
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

export const mockPressureService = new MockPressureService();
