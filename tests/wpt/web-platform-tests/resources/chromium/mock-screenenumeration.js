import {MultipleDisplays, ScreenEnumeration, ScreenEnumerationReceiver} from '/gen/third_party/blink/public/mojom/screen_enumeration/screen_enumeration.mojom.m.js';
import {BufferFormat} from '/gen/ui/gfx/mojom/buffer_types.mojom.m.js';
import {ColorSpaceMatrixID, ColorSpacePrimaryID, ColorSpaceRangeID, ColorSpaceTransferID} from '/gen/ui/gfx/mojom/color_space.mojom.m.js';
import {AccelerometerSupport, Rotation, TouchSupport} from '/gen/ui/display/mojom/display.mojom.m.js';

export const HelperTypes = {
  AccelerometerSupport,
  BufferFormat,
  ColorSpaceMatrixID,
  ColorSpacePrimaryID,
  ColorSpaceRangeID,
  ColorSpaceTransferID,
  Rotation,
  TouchSupport,
};

self.ScreenEnumerationTest = (() => {
  class MockScreenEnumeration {
    constructor() {
      this.receiver_ = new ScreenEnumerationReceiver(this);
      this.interceptor_ =
          new MojoInterfaceInterceptor(ScreenEnumeration.$interfaceName);
      this.interceptor_.oninterfacerequest =
          e => this.receiver_.$.bindHandle(e.handle);
      this.reset();
      this.interceptor_.start();
    }

    reset() {
      this.displays_ = [];
      this.internalId_ = 0;
      this.primaryId_ = 0;
      this.success_ = false;
    }

    setInternalId(internalId) {
      this.internalId_ = internalId;
    }

    setPrimaryId(primaryId) {
      this.primaryId_ = primaryId;
    }

    setSuccess(success) {
      this.success_ = success;
    }

    addDisplay(display) {
      this.displays_.push(display);
    }

    removeDisplay(id) {
      for (var i = 0; i < this.displays_.length; i++) {
        if (this.displays_[i].id === id)
          this.displays_.splice(i,1);
      }
    }

    async getDisplays() {
      if (!this.success_)
        return {result: undefined};
      const result = {
        displays: this.displays_,
        internalId: this.internalId_,
        primaryId: this.primaryId_,
      };
      return {result};
    }

    hasMultipleDisplays() {
      if (!this.success_)
        return {result: MultipleDisplays.kError};
      return {
        result: this.displays_.length > 1
            ? MultipleDisplays.kTrue : MultipleDisplays.kFalse,
      };
    }
  }

  let testInternal = {
    initialized: false,
    mockScreenEnumeration: null
  }

  class ScreenEnumerationTestChromium {
    constructor() {
      Object.freeze(this); // Makes it immutable.
    }

    async initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      if (testInternal.mockScreenEnumeration == null)
        testInternal.mockScreenEnumeration = new MockScreenEnumeration();
      testInternal.initialized = true;
    }

    async reset() {
      if (!testInternal.initialized)
        throw new Error('Call initialize() before reset().');
      testInternal.mockScreenEnumeration.reset();
      testInternal.initialized = false;

      // Wait for an event loop iteration to let any pending mojo commands
      // to finish.
      await new Promise(resolve => setTimeout(resolve, 0));
    }

    getMockScreenEnumeration() {
      return testInternal.mockScreenEnumeration;
    }
  }

  return ScreenEnumerationTestChromium;
})();
