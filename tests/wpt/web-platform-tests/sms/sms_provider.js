let interceptor = (async function() {
  let load = Promise.resolve();
  [
    '/gen/layout_test_data/mojo/public/js/mojo_bindings_lite.js',
    '/gen/mojo/public/mojom/base/big_buffer.mojom-lite.js',
    '/gen/mojo/public/mojom/base/string16.mojom-lite.js',
    '/gen/mojo/public/mojom/base/time.mojom-lite.js',
    '/gen/third_party/blink/public/mojom/sms/sms_manager.mojom-lite.js',
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

class SmsProvider {
  constructor() {
    this.returnValues = {}
  }

  getNextMessage(timeout) {
    let call = this.returnValues.getNextMessage.shift();
    if (!call) {
      throw new Error("Unexpected call.");
    }
    return call(timeout);
  }

  pushReturnValues(callName, returnValues) {
    this.returnValues[callName] = this.returnValues[callName] || [];
    this.returnValues[callName].push(returnValues);
    return this;
  }
}

function getNextMessage(timeout, callback) {
  throw new Error("expected to be overriden by tests");
}

function expect(call) {
  return {
    async andReturn(callback) {
      let provider = await interceptor;
      provider.pushReturnValues(call.name, callback);
    }
  }
}

const Status = {};

function intercept() {
  let provider = new SmsProvider();

  let interceptor = new MojoInterfaceInterceptor(
      blink.mojom.SmsManager.$interfaceName);
  interceptor.oninterfacerequest = (e) => {
    let impl = new blink.mojom.SmsManager(provider);
    impl.bindHandle(e.handle);
  }

  interceptor.start();

  Status.kSuccess = blink.mojom.SmsStatus.kSuccess;
  Status.kTimeout = blink.mojom.SmsStatus.kTimeout;

  return provider;
}
