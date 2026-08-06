'use strict';

import {SubAppsService, SubAppsServiceReceiver, SubAppsServiceResultCode, SubAppsServiceAddResultType, SubAppsServiceRemoveResultType} from '/gen/third_party/blink/public/mojom/subapps/sub_apps_service.mojom.m.js';

self.SubAppsServiceAddResultType = SubAppsServiceAddResultType;
self.SubAppsServiceRemoveResultType = SubAppsServiceRemoveResultType;
self.SubAppsServiceResultCode = SubAppsServiceResultCode;

self.SubAppsServiceTest = (() => {
  // Class that mocks SubAppsService interface defined in /third_party/blink/public/mojom/subapps/sub_apps_service.mojom

  class MockSubAppsService {
    constructor() {
      this.interceptor_ =
        new MojoInterfaceInterceptor(SubAppsService.$interfaceName);
      this.receiver_ = new SubAppsServiceReceiver(this);
      this.interceptor_.oninterfacerequest =
        e => this.receiver_.$.bindHandle(e.handle);
      this.interceptor_.start();
    }

    reset() {
      this.interceptor_.stop();
      this.receiver_.$.close();
    }

    add(install_urls) {
      if (testInternal.serviceResultCode === -1) {
        return Promise.resolve(testInternal.addCallReturnValue);
      }
      throw testInternal.serviceResultCode;
    }

    list() {
      if (testInternal.serviceResultCode === -1) {
        return Promise.resolve(testInternal.listCallReturnValue);
      }
      throw testInternal.serviceResultCode;
    }

    remove(manifest_ids) {
      if (testInternal.serviceResultCode === -1) {
        return Promise.resolve(testInternal.removeCallReturnValue);
      }
      throw testInternal.serviceResultCode;
    }
  }

  let testInternal = {
    initialized: false,
    mockSubAppsService: null,
    serviceResultCode: -1,
    addCallReturnValue: [],
    listCallReturnValue: [],
    removeCallReturnValue: [],
  }

  class SubAppsServiceTestChromium {
    constructor() {
      Object.freeze(this);  // Make it immutable.
    }

    initialize(service_result_code, add_call_return_value, list_call_return_value, remove_call_return_value) {
      if (!testInternal.initialized) {
        testInternal = {
          mockSubAppsService: new MockSubAppsService(),
          initialized: true,
          serviceResultCode: service_result_code,
          addCallReturnValue: add_call_return_value,
          listCallReturnValue: list_call_return_value,
          removeCallReturnValue: remove_call_return_value,
        };
      };
    }

    setAddCallReturnValue(value) {
      testInternal.addCallReturnValue = value;
    }

    setListCallReturnValue(value) {
      testInternal.listCallReturnValue = value;
    }

    setRemoveCallReturnValue(value) {
      testInternal.removeCallReturnValue = value;
    }

    async reset() {
      if (testInternal.initialized) {
        testInternal.mockSubAppsService.reset();
        testInternal = {
          mockSubAppsService: null,
          initialized: false,
          serviceResultCode: -1,
          addCallReturnValue: [],
          listCallReturnValue: [],
          removeCallReturnValue: [],
        };
        await new Promise(resolve => setTimeout(resolve, 0));
      }
    }
  }

  return SubAppsServiceTestChromium;
})();
