import {BatteryMonitor, BatteryMonitorReceiver} from '/gen/services/device/public/mojom/battery_monitor.mojom.m.js';

class MockBatteryMonitor {
  constructor() {
    this.receiver_ = new BatteryMonitorReceiver(this);
    this.interceptor_ =
        new MojoInterfaceInterceptor(BatteryMonitor.$interfaceName);
    this.interceptor_.oninterfacerequest = e =>
        this.receiver_.$.bindHandle(e.handle);
    this.reset();
  }

  start() {
    this.interceptor_.start();
  }

  stop() {
    this.interceptor_.stop();
  }

  reset() {
    this.pendingRequests_ = [];
    this.status_ = null;
    this.lastKnownStatus_ = null;
  }

  queryNextStatus() {
    const result = new Promise(resolve => this.pendingRequests_.push(resolve));
    this.runCallbacks_();
    return result;
  }

  setBatteryStatus(charging, chargingTime, dischargingTime, level) {
    this.status_ = {charging, chargingTime, dischargingTime, level};
    this.lastKnownStatus_ = this.status_;
    this.runCallbacks_();
  }

  verifyBatteryStatus(manager) {
    assert_not_equals(manager, undefined);
    assert_not_equals(this.lastKnownStatus_, null);
    assert_equals(manager.charging, this.lastKnownStatus_.charging);
    assert_equals(manager.chargingTime, this.lastKnownStatus_.chargingTime);
    assert_equals(
        manager.dischargingTime, this.lastKnownStatus_.dischargingTime);
    assert_equals(manager.level, this.lastKnownStatus_.level);
  }

  runCallbacks_() {
    if (!this.status_ || !this.pendingRequests_.length)
      return;

    let result = {status: this.status_};
    while (this.pendingRequests_.length) {
      this.pendingRequests_.pop()(result);
    }
    this.status_ = null;
  }
}

export const mockBatteryMonitor = new MockBatteryMonitor();
