import {PressureManager, PressureManagerReceiver, PressureStatus} from '/gen/services/device/public/mojom/pressure_manager.mojom.m.js'
import {PressureFactor, PressureState} from '/gen/services/device/public/mojom/pressure_update.mojom.m.js'

class MockPressureService {
  constructor() {
    this.receiver_ = new PressureManagerReceiver(this);
    this.interceptor_ =
        new MojoInterfaceInterceptor(PressureManager.$interfaceName);
    this.interceptor_.oninterfacerequest = e => {
      this.receiver_.$.bindHandle(e.handle);
    };
    this.receiver_.onConnectionError.addListener(() => {
      this.stopPlatformCollector();
      this.observer_ = null;
    });
    this.reset();
    this.mojomStateType_ = new Map([
      ['nominal', PressureState.kNominal], ['fair', PressureState.kFair],
      ['serious', PressureState.kSerious], ['critical', PressureState.kCritical]
    ]);
    this.mojomFactorType_ = new Map([
      ['thermal', PressureFactor.kThermal],
      ['power-supply', PressureFactor.kPowerSupply]
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
    this.observer_ = null;
    this.pressureUpdate_ = null;
    this.pressureServiceReadingTimerId_ = null;
    this.pressureStatus_ = PressureStatus.kOk;
    this.updatesDelivered_ = 0;
  }

  async addClient(observer) {
    if (this.observer_ !== null)
      throw new Error('BindObserver() has already been called');

    this.observer_ = observer;
    this.observer_.onConnectionError.addListener(() => {
      this.stopPlatformCollector();
      this.observer_ = null;
    });

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
    this.pressureServiceReadingTimerId_ = window.setInterval(() => {
      if (this.pressureUpdate_ === null || this.observer_ === null)
        return;
      this.pressureUpdate_.timestamp = {
        internalValue: BigInt((new Date().getTime() + epochDeltaInMs) * 1000)
      };
      this.observer_.onPressureUpdated(this.pressureUpdate_);
      this.updatesDelivered_++;
    }, timeout);
  }

  stopPlatformCollector() {
    if (this.pressureServiceReadingTimerId_ != null) {
      window.clearInterval(this.pressureServiceReadingTimerId_);
      this.pressureServiceReadingTimerId_ = null;
    }
    this.updatesDelivered_ = 0;
  }

  updatesDelivered() {
    return this.updatesDelivered_;
  }

  setPressureUpdate(state, factors) {
    if (!this.mojomStateType_.has(state))
      throw new Error(`PressureState '${state}' is invalid`);

    let pressureFactors = [];
    if (Array.isArray(factors)) {
      for (const factor of factors) {
        if (!this.mojomFactorType_.has(factor))
          throw new Error(`PressureFactor '${factor}' is invalid`);
        pressureFactors.push(this.mojomFactorType_.get(factor));
      }
    }

    this.pressureUpdate_ = {
      state: this.mojomStateType_.get(state),
      factors: pressureFactors,
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
