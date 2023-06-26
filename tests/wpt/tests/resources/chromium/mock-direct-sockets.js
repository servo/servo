'use strict';

import {DirectSocketsService, DirectSocketsServiceReceiver} from '/gen/third_party/blink/public/mojom/direct_sockets/direct_sockets.mojom.m.js';

self.DirectSocketsServiceTest = (() => {
  // Class that mocks DirectSocketsService interface defined in
  // https://source.chromium.org/chromium/chromium/src/third_party/blink/public/mojom/direct_sockets/direct_sockets.mojom
  class MockDirectSocketsService {
    constructor() {
      this.interceptor_ = new MojoInterfaceInterceptor(DirectSocketsService.$interfaceName);
      this.receiver_ = new DirectSocketsServiceReceiver(this);
      this.interceptor_.oninterfacerequest = e =>
          this.receiver_.$.bindHandle(e.handle);
      this.interceptor_.start();
    }

    reset() {
      this.receiver_.$.close();
      this.interceptor_.stop();
    }

    openTCPSocket(
      options,
      receiver,
      observer) {
      return Promise.resolve({
        // return result = net:Error::NOT_IMPLEMENTED (code -11)
        result: -11
      });
    }

    openConnectedUDPSocket(
      options,
      receiver,
      listener) {
      return Promise.resolve({
        // return result = net:Error::NOT_IMPLEMENTED (code -11)
        result: -11
      });
    }

    openBoundUDPSocket(
      options,
      receiver,
      listener) {
      return Promise.resolve({
        // return result = net:Error::NOT_IMPLEMENTED (code -11)
        result: -11
      });
    }

    openTCPServerSocket(
      options,
      receiver) {
      return Promise.resolve({
        // return result = net:Error::NOT_IMPLEMENTED (code -11)
        result: -11
      });
    }
  }

  let testInternal = {
    initialized: false,
    mockDirectSocketsService: null
  }

  class DirectSocketsServiceTestChromium {
    constructor() {
      Object.freeze(this);  // Make it immutable.
    }

    initialize() {
      if (!testInternal.initialized) {
        testInternal = {
          mockDirectSocketsService: new MockDirectSocketsService(),
          initialized: true
        };
      }
    }

    async reset() {
      if (testInternal.initialized) {
        testInternal.mockDirectSocketsService.reset();
        testInternal = {
          mockDirectSocketsService: null,
          initialized: false
        };
        await new Promise(resolve => setTimeout(resolve, 0));
      }
    }
  }

  return DirectSocketsServiceTestChromium;
})();
