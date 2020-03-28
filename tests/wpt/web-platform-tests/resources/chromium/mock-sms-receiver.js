'use strict';

const SmsProvider = (() => {

  class MockSmsReceiver {

    constructor() {
      this.mojoReceiver_ = new blink.mojom.SmsReceiverReceiver(this);

      this.interceptor_ =
          new MojoInterfaceInterceptor(blink.mojom.SmsReceiver.$interfaceName);

      this.interceptor_.oninterfacerequest = (e) => {
        this.mojoReceiver_.$.bindHandle(e.handle);
      }
      this.interceptor_.start();

      this.returnValues_ = {};
    }

    async receive() {
      let call = this.returnValues_.receive ?
          this.returnValues_.receive.shift() : null;
      if (!call)
        return;
      return call();
    }

    async abort() {};

    pushReturnValuesForTesting(callName, value) {
      this.returnValues_[callName] = this.returnValues_[callName] || [];
      this.returnValues_[callName].push(value);
      return this;
    }
  }

  const mockSmsReceiver = new MockSmsReceiver();

  class SmsProviderChromium {
    constructor() {
      Object.freeze(this); // Make it immutable.
    }

    pushReturnValuesForTesting(callName, callback) {
      mockSmsReceiver.pushReturnValuesForTesting(callName, callback);
    }
  }

  return SmsProviderChromium;
})();
