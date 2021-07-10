// These tests rely on the User Agent providing an implementation of
// MockWebOTPService.
//
// In Chromium-based browsers this implementation is provided by a polyfill
// in order to reduce the amount of test-only code shipped to users. To enable
// these tests the browser must be run with these options:
// //   --enable-blink-features=MojoJS,MojoJSTest

import {isChromiumBased} from '/resources/test-only-api.m.js';

/**
 * This enumeration is used by WebOTP WPTs to control mock backend behavior.
 * See MockWebOTPService below.
 */
export const Status = {
  SUCCESS: 0,
  UNHANDLED_REQUEST: 1,
  CANCELLED: 2,
  ABORTED: 3,
};

/**
 * A interface which must be implemented by browsers to support WebOTP WPTs.
 */
export class MockWebOTPService {
  /**
   * Accepts a function to be invoked in response to the next OTP request
   * received by the mock. The (optionally async) function, when executed, must
   * return an object with a `status` field holding one of the `Status` values
   * defined above, and -- if successful -- an `otp` field containing a
   * simulated OTP string.
   *
   * Tests will call this method directly to inject specific response behavior
   * into the browser-specific mock implementation.
   */
  async handleNextOTPRequest(responseFunc) {}
}

/**
 * Returns a Promise resolving to a browser-specific MockWebOTPService subclass
 * instance if one is available.
 */
async function createBrowserSpecificMockImpl() {
  if (isChromiumBased) {
    return await createChromiumMockImpl();
  }
  throw new Error('Unsupported browser.');
}

const asyncMock = createBrowserSpecificMockImpl();

export function expectOTPRequest() {
  return {
    async andReturn(callback) {
      const mock = await asyncMock;
      mock.handleNextOTPRequest(callback);
    }
  }
}

/**
 * Instantiates a Chromium-specific subclass of MockWebOTPService.
 */
async function createChromiumMockImpl() {
  const {SmsStatus, WebOTPService, WebOTPServiceReceiver} = await import(
      '/gen/third_party/blink/public/mojom/sms/webotp_service.mojom.m.js');
  const MockWebOTPServiceChromium = class extends MockWebOTPService {
    constructor() {
      super();
      this.mojoReceiver_ = new WebOTPServiceReceiver(this);
      this.interceptor_ =
          new MojoInterfaceInterceptor(WebOTPService.$interfaceName);
      this.interceptor_.oninterfacerequest = (e) => {
        this.mojoReceiver_.$.bindHandle(e.handle);
      };
      this.interceptor_.start();
      this.requestHandlers_ = [];
      Object.freeze(this);
    }

    handleNextOTPRequest(responseFunc) {
      this.requestHandlers_.push(responseFunc);
    }

    async receive() {
      if (this.requestHandlers_.length == 0) {
        throw new Error('Mock received unexpected OTP request.');
      }

      const responseFunc = this.requestHandlers_.shift();
      const response = await responseFunc();
      switch (response.status) {
        case Status.SUCCESS:
          if (typeof response.otp != 'string') {
            throw new Error('Mock success results require an OTP string.');
          }
          return {status: SmsStatus.kSuccess, otp: response.otp};
        case Status.UNHANDLED_REQUEST:
          return {status: SmsStatus.kUnhandledRequest};
        case Status.CANCELLED:
          return {status: SmsStatus.kCancelled};
        case Status.ABORTED:
          return {status: SmsStatus.kAborted};
        default:
          throw new Error(
              `Mock result contains unknown status: ${response.status}`);
      }
    }

    async abort() {}
  };
  return new MockWebOTPServiceChromium();
}

