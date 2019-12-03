'use strict';

function toMojoNDEFPushTarget(target) {
  switch (target) {
  case 'peer':
    return device.mojom.NDEFPushTarget.PEER;
  case 'tag':
    return device.mojom.NDEFPushTarget.TAG;
  }

  return device.mojom.NDEFPushTarget.ANY;
}

// Converts between NDEFMessageInit https://w3c.github.io/web-nfc/#dom-ndefmessage
// and mojom.NDEFMessage structure, so that watch function can be tested.
function toMojoNDEFMessage(message) {
  let ndefMessage = new device.mojom.NDEFMessage();
  ndefMessage.url = message.url;
  ndefMessage.data = [];
  for (let record of message.records) {
    ndefMessage.data.push(toMojoNDEFRecord(record));
  }
  return ndefMessage;
}

function toMojoNDEFRecord(record) {
  let nfcRecord = new device.mojom.NDEFRecord();
  nfcRecord.recordType = record.recordType;
  nfcRecord.mediaType = record.mediaType;
  nfcRecord.id = record.id;
  nfcRecord.data = toByteArray(record.data);
  if (record.data != null && record.data.records !== undefined) {
    // |record.data| may be an NDEFMessageInit, i.e. the payload is a message.
    nfcRecord.payloadMessage = toMojoNDEFMessage(record.data);
  }
  return nfcRecord;
}

// Converts JS objects to byte array.
function toByteArray(data) {
  if (data instanceof ArrayBuffer)
    return new Uint8Array(data);
  else if (ArrayBuffer.isView(data))
    return new Uint8Array(data.buffer, data.byteOffset, data.byteLength);

  let byteArray = new Uint8Array(0);
  let tmpData = data;
  if (typeof tmpData === 'object' || typeof tmpData === 'number')
    tmpData = JSON.stringify(tmpData);

  if (typeof tmpData === 'string')
    byteArray = new TextEncoder('utf-8').encode(tmpData);

  return byteArray;
}

// Compares NDEFRecords that were provided / received by the mock service.
// TODO: Use different getters to get received record data,
// see spec changes at https://github.com/w3c/web-nfc/pull/243.
function compareNDEFRecords(providedRecord, receivedRecord) {
  assert_equals(providedRecord.recordType, receivedRecord.recordType);

  if (providedRecord.id === undefined) {
    assert_equals(null, receivedRecord.id);
  } else {
    assert_equals(providedRecord.id, receivedRecord.id);
  }

  if (providedRecord.mediaType === undefined) {
    assert_equals(null, receivedRecord.mediaType);
  } else {
    assert_equals(providedRecord.mediaType, receivedRecord.mediaType);
  }

  assert_not_equals(providedRecord.recordType, 'empty');

  assert_array_equals(toByteArray(providedRecord.data),
                      new Uint8Array(receivedRecord.data));
}

// Compares NDEFPushOptions structures that were provided to API and
// received by the mock mojo service.
function assertNDEFPushOptionsEqual(provided, received) {
  if (provided.ignoreRead !== undefined)
    assert_equals(provided.ignoreRead, !!received.ignoreRead);
  else
    assert_equals(!!received.ignore_read, true);

  if (provided.target !== undefined)
    assert_equals(toMojoNDEFPushTarget(provided.target), received.target);
  else
    assert_equals(received.target, device.mojom.NDEFPushTarget.ANY);
}

// Compares NDEFReaderOptions structures that were provided to API and
// received by the mock mojo service.
function assertNDEFReaderOptionsEqual(provided, received) {
  if (provided.url !== undefined)
    assert_equals(provided.url, received.url);
  else
    assert_equals(received.url, '');

  if (provided.mediaType !== undefined)
    assert_equals(provided.mediaType, received.mediaType);
  else
    assert_equals(received.mediaType, '');

  if (provided.recordType !== undefined) {
    assert_equals(provided.recordType, received.recordType);
  }
}

// Checks whether NDEFReaderOptions are matched with given message.
function matchesWatchOptions(message, options) {
  // Filter by Web NFC id.
  if (!matchesWebNfcId(message.url, options.id)) return false;

  // Matches any record / media type.
  if ((options.mediaType == null || options.mediaType === '') &&
      options.recordType == null) {
    return true;
  }

  // Filter by mediaType and recordType.
  for (let record of message.records) {
    if (options.mediaType != null && options.mediaType !== ""
        && options.mediaType !== record.mediaType) {
      return false;
    }
    if (options.recordType != null &&
        options.recordType !== record.recordType) {
      return false;
    }
  }

  return true;
}

// Web NFC id match algorithm.
// https://w3c.github.io/web-nfc/#url-pattern-match-algorithm
function matchesWebNfcId(id, pattern) {
  if (id != null && id !== "" && pattern != null && pattern !== "") {
    const id_url = new URL(id);
    const pattern_url = new URL(pattern);

    if (id_url.protocol !== pattern_url.protocol) return false;
    if (!id_url.host.endsWith("." + pattern_url.host)
        && id_url.host !== pattern_url.host) {
      return false;
    }
    if (pattern_url.pathname === "/*") return true;
    if (id_url.pathname.startsWith(pattern_url.pathname)) return true;

    return false;
  }

  return true;
}

function createNDEFError(type) {
  return { error: type ?
      new device.mojom.NDEFError({ errorType: type }) : null };
}

var WebNFCTest = (() => {
  class MockNFC {
    constructor() {
      this.bindingSet_ = new mojo.BindingSet(device.mojom.NFC);

      this.interceptor_ = new MojoInterfaceInterceptor(
          device.mojom.NFC.name, "context", true);
      this.interceptor_.oninterfacerequest =
          e => this.bindingSet_.addBinding(this, e.handle);
      this.interceptor_.start();

      this.hw_status_ = NFCHWStatus.ENABLED;
      this.pushed_message_ = null;
      this.push_options_ = null;
      this.pending_promise_func_ = null;
      this.push_completed_ = true;
      this.client_ = null;
      this.watchers_ = [];
      this.reading_messages_ = [];
      this.operations_suspended_ = false;
      this.is_ndef_tech_ = true;
      this.is_formatted_tag_ = false;
    }

    // NFC delegate functions.
    async push(message, options) {
      let error = this.getHWError();
      if (error)
        return error;
      // Cancels previous pending push operation.
      if (this.pending_promise_func_) {
        this.cancelPendingPushOperation();
      }

      this.pushed_message_ = message;
      this.push_options_ = options;

      return new Promise(resolve => {
        this.pending_promise_func_ = resolve;
        if (this.operations_suspended_) {
          // Pends push operation if NFC operation is suspended.
        } else if (!this.push_completed_) {
          // Leaves the push operating pending.
        } else if (!this.is_ndef_tech_) {
          // Resolves with NotSupportedError if the device does not expose
          // NDEF technology.
          resolve(createNDEFError(device.mojom.NDEFErrorType.NOT_SUPPORTED));
        } else if (this.is_formatted_tag_ && !options.overwrite) {
          // Resolves with NotAllowedError if there are NDEF records on the device
          // and overwrite is false.
          resolve(createNDEFError(device.mojom.NDEFErrorType.NOT_ALLOWED));
        } else {
          resolve(createNDEFError(null));
        }
      });
    }

    async cancelPush(target) {
      if (this.push_options_ && ((target === device.mojom.NDEFPushTarget.ANY) ||
          (this.push_options_.target === target))) {
        this.cancelPendingPushOperation();
      }

      return createNDEFError(null);
    }

    setClient(client) {
      this.client_ = client;
    }

    async watch(options, id) {
      assert_true(id > 0);
      let error = this.getHWError();
      if (error) {
        return error;
      }

      this.watchers_.push({id: id, options: options});
      // Ignores reading if NFC operation is suspended
      // or the NFC tag does not expose NDEF technology.
      if(!this.operations_suspended_ && this.is_ndef_tech_) {
        // Triggers onWatch if the new watcher matches existing messages.
        for (let message of this.reading_messages_) {
          if (matchesWatchOptions(message, options)) {
            this.client_.onWatch(
                [id], fake_tag_serial_number, toMojoNDEFMessage(message));
          }
        }
      }

      return createNDEFError(null);
    }

    async cancelWatch(id) {
      let index = this.watchers_.findIndex(value => value.id === id);
      if (index === -1) {
        return createNDEFError(device.mojom.NDEFErrorType.NOT_FOUND);
      }

      this.watchers_.splice(index, 1);
      return createNDEFError(null);
    }

    async cancelAllWatches() {
      if (this.watchers_.length === 0) {
        return createNDEFError(device.mojom.NDEFErrorType.NOT_FOUND);
      }

      this.watchers_.splice(0, this.watchers_.length);
      return createNDEFError(null);
    }

    getHWError() {
      if (this.hw_status_ === NFCHWStatus.DISABLED)
        return createNDEFError(device.mojom.NDEFErrorType.NOT_READABLE);
      if (this.hw_status_ === NFCHWStatus.NOT_SUPPORTED)
        return createNDEFError(device.mojom.NDEFErrorType.NOT_SUPPORTED);
      return null;
    }

    setHWStatus(status) {
      this.hw_status_ = status;
    }

    pushedMessage() {
      return this.pushed_message_;
    }

    pushOptions() {
      return this.push_options_;
    }

    watchOptions() {
      assert_not_equals(this.watchers_.length, 0);
      return this.watchers_[this.watchers_.length - 1].options;
    }

    setPendingPushCompleted(result) {
      this.push_completed_ = result;
    }

    reset() {
      this.hw_status_ = NFCHWStatus.ENABLED;
      this.watchers_ = [];
      this.reading_messages_ = [];
      this.operations_suspended_ = false;
      this.cancelPendingPushOperation();
      this.is_ndef_tech_ = true;
      this.is_formatted_tag_ = false;
    }

    cancelPendingPushOperation() {
      if (this.pending_promise_func_) {
        this.pending_promise_func_(
            createNDEFError(device.mojom.NDEFErrorType.OPERATION_CANCELLED));
      }

      this.pushed_message_ = null;
      this.push_options_ = null;
      this.pending_promise_func_ = null;
      this.push_completed_ = true;
    }

    // Sets message that is used to deliver NFC reading updates.
    setReadingMessage(message) {
      this.reading_messages_.push(message);
      // Ignores reading if the NFC tag does not expose NDEF technology.
      if(!this.is_ndef_tech_) return;
      // Ignores reading if NFC operation is suspended.
      if(this.operations_suspended_) return;
      // Ignores reading if NDEFPushOptions.ignoreRead is true.
      if(this.push_options_ && this.push_options_.ignoreRead)
        return;
      // Triggers onWatch if the new message matches existing watchers.
      for (let watcher of this.watchers_) {
        if (matchesWatchOptions(message, watcher.options)) {
          this.client_.onWatch(
              [watcher.id], fake_tag_serial_number,
              toMojoNDEFMessage(message));
        }
      }
    }

    // Suspends all pending NFC operations. Could be used when web page
    // visibility is lost.
    suspendNFCOperations() {
      this.operations_suspended_ = true;
    }

    // Resumes all suspended NFC operations.
    resumeNFCOperations() {
      this.operations_suspended_ = false;
      // Resumes pending NFC reading.
      for (let watcher of this.watchers_) {
        for (let message of this.reading_messages_) {
          if (matchesWatchOptions(message, watcher.options) && this.is_ndef_tech_) {
            this.client_.onWatch(
                [watcher.id], fake_tag_serial_number,
                toMojoNDEFMessage(message));
          }
        }
      }
      // Resumes pending push operation.
      if (this.pending_promise_func_) {
        this.pending_promise_func_(createNDEFError(null));
      }
    }

    setIsNDEFTech(isNdef) {
      this.is_ndef_tech_ = isNdef;
    }

    setIsFormattedTag(isFormatted) {
      this.is_formatted_tag_ = isFormatted;
    }
  }

  let testInternal = {
    initialized: false,
    mockNFC: null
  }

  class NFCTestChromium {
    constructor() {
      Object.freeze(this); // Makes it immutable.
    }

    initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      if (window.testRunner) {
        // Grant nfc permissions for Chromium testrunner.
        window.testRunner.setPermission('nfc', 'granted',
                                        location.origin, location.origin);
      }

      if (testInternal.mockNFC == null) {
        testInternal.mockNFC = new MockNFC();
      }
      testInternal.initialized = true;
    }

    // Reuses the nfc mock but resets its state between test runs.
    async reset() {
      if (!testInternal.initialized)
        throw new Error('Call initialize() before reset().');
      testInternal.mockNFC.reset();
      testInternal.initialized = false;

      await new Promise(resolve => setTimeout(resolve, 0));
    }

    getMockNFC() {
      return testInternal.mockNFC;
    }
  }

  return NFCTestChromium;
})();
