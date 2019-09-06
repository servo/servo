let interceptor = (async function() {
  let load = Promise.resolve();
  [
    '/gen/layout_test_data/mojo/public/js/mojo_bindings_lite.js',
    '/gen/mojo/public/mojom/base/big_buffer.mojom-lite.js',
    '/gen/mojo/public/mojom/base/string16.mojom-lite.js',
    '/gen/mojo/public/mojom/base/time.mojom-lite.js',
    '/gen/third_party/blink/public/mojom/sms/sms_receiver.mojom-lite.js',
  ].forEach(path => {
    let script = document.createElement('script');
    script.src = path;
    script.async = false;
    load = load.then(() => new Promise(resolve => {
      script.onload = resolve;
    }));
    document.head.appendChild(script);
  });

  return load.then(intercept);
})();

// Fake implementation of blink.mojom.SmsReceiver.
class FakeSmsReceiverImpl {
  constructor() {
    this.returnValues = {}
  }

  bindHandleToMojoReceiver(handle) {
    this.mojoReceiver_ = new blink.mojom.SmsReceiverReceiver(this);
    this.mojoReceiver_.$.bindHandle(handle);
  }

  pushReturnValuesForTesting(callName, returnValues) {
    this.returnValues[callName] = this.returnValues[callName] || [];
    this.returnValues[callName].push(returnValues);
    return this;
  }

  receive() {
    let call = this.returnValues.receive.shift();
    if (!call) {
      throw new Error("Unexpected call.");
    }
    return call();
  }
}

function receive(callback) {
  throw new Error("expected to be overriden by tests");
}

function expect(call) {
  return {
    async andReturn(callback) {
      let smsReceiverImpl = await interceptor;
      smsReceiverImpl.pushReturnValuesForTesting(call.name, callback);
    }
  }
}

const Status = {};

function intercept() {
  let smsReceiverImpl = new FakeSmsReceiverImpl();

  let interceptor = new MojoInterfaceInterceptor(
      blink.mojom.SmsReceiver.$interfaceName);
  interceptor.oninterfacerequest = (e) => {
    smsReceiverImpl.bindHandleToMojoReceiver(e.handle);
  }

  interceptor.start();

  Status.kSuccess = blink.mojom.SmsStatus.kSuccess;
  Status.kTimeout = blink.mojom.SmsStatus.kTimeout;
  Status.kCancelled = blink.mojom.SmsStatus.kCancelled;

  return smsReceiverImpl;
}
