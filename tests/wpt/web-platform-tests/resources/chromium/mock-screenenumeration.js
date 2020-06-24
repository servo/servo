'use strict'

var ScreenEnumerationTest = (() => {

  class MockScreenEnumeration {
    constructor() {
      this.bindingSet_ = new mojo.BindingSet(blink.mojom.ScreenEnumeration);
      this.interceptor_ = new MojoInterfaceInterceptor(blink.mojom.ScreenEnumeration.name);
      this.interceptor_.oninterfacerequest = e => {
        this.bindingSet_.addBinding(this, e.handle);
      }
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
      return Promise.resolve({
        displays: this.displays_,
        internalId: this.internalId_,
        primaryId: this.primaryId_,
        success: this.success_,
      });
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
