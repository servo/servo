'use strict';

// These tests rely on the User Agent providing an implementation of
// the sms retriever.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
// //   --enable-blink-features=MojoJS,MojoJSTest

const Status = {};

async function loadChromiumResources() {
  const resources = [
    '/gen/mojo/public/mojom/base/time.mojom-lite.js',
    '/gen/third_party/blink/public/mojom/sms/webotp_service.mojom-lite.js',
  ];

  await loadMojoResources(resources, true);
  await loadScript('/resources/chromium/mock-sms-receiver.js');

  Status.kSuccess = blink.mojom.SmsStatus.kSuccess;
  Status.kTimeout = blink.mojom.SmsStatus.kTimeout;
  Status.kCancelled = blink.mojom.SmsStatus.kCancelled;
};

async function create_sms_provider() {
  if (typeof SmsProvider === 'undefined') {
    if (isChromiumBased) {
      await loadChromiumResources();
    } else {
      throw new Error('Mojo testing interface is not available.');
    }
  }
  if (typeof SmsProvider === 'undefined') {
    throw new Error('Failed to set up SmsProvider.');
  }
  return new SmsProvider();
}

function receive() {
  throw new Error("expected to be overriden by tests");
}

function expect(call) {
  return {
    async andReturn(callback) {
      const mock = await create_sms_provider();
      mock.pushReturnValuesForTesting(call.name, callback);
    }
  }
}
