'use strict';

// Converts between NDEFMessageInit https://w3c.github.io/web-nfc/#dom-ndefmessage
// and mojom.NDEFMessage structure, so that watch function can be tested.
function toMojoNDEFMessage(message) {
  let ndefMessage = new device.mojom.NDEFMessage();
  ndefMessage.data = [];
  for (let record of message.records) {
    ndefMessage.data.push(toMojoNDEFRecord(record));
  }
  return ndefMessage;
}

function toMojoNDEFRecord(record) {
  let nfcRecord = new device.mojom.NDEFRecord();
  // Simply checks the existence of ':' to decide whether it's an external
  // type or a local type. As a mock, no need to really implement the validation
  // algorithms for them.
  if (record.recordType.startsWith(':')) {
    nfcRecord.category = device.mojom.NDEFRecordTypeCategory.kLocal;
  } else if (record.recordType.search(':') != -1) {
    nfcRecord.category = device.mojom.NDEFRecordTypeCategory.kExternal;
  } else {
    nfcRecord.category = device.mojom.NDEFRecordTypeCategory.kStandardized;
  }
  nfcRecord.recordType = record.recordType;
  nfcRecord.mediaType = record.mediaType;
  nfcRecord.id = record.id;
  if (record.recordType == 'text') {
    nfcRecord.encoding = record.encoding == null? 'utf-8': record.encoding;
    nfcRecord.lang = record.lang == null? 'en': record.lang;
  }
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

  if (providedRecord.recordType == 'text') {
    assert_equals(
        providedRecord.encoding == null? 'utf-8': providedRecord.encoding,
        receivedRecord.encoding);
    assert_equals(providedRecord.lang == null? 'en': providedRecord.lang,
                  receivedRecord.lang);
  } else {
    assert_equals(null, receivedRecord.encoding);
    assert_equals(null, receivedRecord.lang);
  }

  assert_array_equals(toByteArray(providedRecord.data),
                      new Uint8Array(receivedRecord.data));
}

// Compares NDEFWriteOptions structures that were provided to API and
// received by the mock mojo service.
function assertNDEFWriteOptionsEqual(provided, received) {
  if (provided.ignoreRead !== undefined)
    assert_equals(provided.ignoreRead, !!received.ignoreRead);
  else
    assert_equals(!!received.ignore_read, true);
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
  // A message with no records is to notify that the tag is already formatted to
  // support NDEF but does not contain a message yet. We always dispatch it for
  // all options.
  if (message.records.length == 0)
    return true;

  for (let record of message.records) {
    if (options.id != null && options.id !== record.id) {
      continue;
    }
    if (options.recordType != null &&
        options.recordType !== record.recordType) {
      continue;
    }
    if (options.mediaType != null && options.mediaType !== record.mediaType) {
      continue;
    }

    // Found one record matches, means the message matches.
    return true;
  }

  return false;
}

function createNDEFError(type) {
  return {
    error: type != null ?
        new device.mojom.NDEFError({errorType: type, errorMessage: ''}) :
        null
  };
}

var WebNFCTest = (() => {
  class MockNFC {
    constructor() {
      this.bindingSet_ = new mojo.BindingSet(device.mojom.NFC);

      this.interceptor_ = new MojoInterfaceInterceptor(device.mojom.NFC.name);
      this.interceptor_.oninterfacerequest = e => {
        if (this.should_close_pipe_on_request_)
          e.handle.close();
        else
          this.bindingSet_.addBinding(this, e.handle);
      }

      this.interceptor_.start();

      this.hw_status_ = NFCHWStatus.ENABLED;
      this.pushed_message_ = null;
      this.pending_write_options_ = null;
      this.pending_promise_func_ = null;
      this.push_completed_ = true;
      this.client_ = null;
      this.watchers_ = [];
      this.reading_messages_ = [];
      this.operations_suspended_ = false;
      this.is_formatted_tag_ = false;
      this.data_transfer_failed_ = false;
      this.should_close_pipe_on_request_ = false;
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
      this.pending_write_options_ = options;
      return new Promise(resolve => {
        if (this.operations_suspended_ || !this.push_completed_) {
          // Leaves the push pending.
          this.pending_promise_func_ = resolve;
        } else if (this.is_formatted_tag_ && !options.overwrite) {
          // Resolves with NotAllowedError if there are NDEF records on the device
          // and overwrite is false.
          resolve(createNDEFError(device.mojom.NDEFErrorType.NOT_ALLOWED));
        } else if (this.data_transfer_failed_) {
          // Resolves with NetworkError if data transfer fails.
          resolve(createNDEFError(device.mojom.NDEFErrorType.IO_ERROR));
        } else {
          resolve(createNDEFError(null));
        }
      });
    }

    async cancelPush() {
      this.cancelPendingPushOperation();
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
      if (!this.operations_suspended_) {
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

    writeOptions() {
      return this.pending_write_options_;
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
      this.is_formatted_tag_ = false;
      this.data_transfer_failed_ = false;
      this.should_close_pipe_on_request_ = false;
    }

    cancelPendingPushOperation() {
      if (this.pending_promise_func_) {
        this.pending_promise_func_(
            createNDEFError(device.mojom.NDEFErrorType.OPERATION_CANCELLED));
        this.pending_promise_func_ = null;
      }

      this.pushed_message_ = null;
      this.pending_write_options_ = null;
      this.push_completed_ = true;
    }

    // Sets message that is used to deliver NFC reading updates.
    setReadingMessage(message) {
      this.reading_messages_.push(message);
      // Ignores reading if NFC operation is suspended.
      if(this.operations_suspended_) return;
      // Ignores reading if NDEFWriteOptions.ignoreRead is true.
      if (this.pending_write_options_ && this.pending_write_options_.ignoreRead)
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
          if (matchesWatchOptions(message, watcher.options)) {
            this.client_.onWatch(
                [watcher.id], fake_tag_serial_number,
                toMojoNDEFMessage(message));
          }
        }
      }
      // Resumes pending push operation.
      if (this.pending_promise_func_ && this.push_completed_) {
        this.pending_promise_func_(createNDEFError(null));
        this.pending_promise_func_ = null;
      }
    }

    // Simulates the device coming in proximity does not expose NDEF technology.
    simulateNonNDEFTagDiscovered() {
      // Notify NotSupportedError to all active readers.
      if (this.watchers_.length != 0) {
        this.client_.onError(new device.mojom.NDEFError({
          errorType: device.mojom.NDEFErrorType.NOT_SUPPORTED,
          errorMessage: ''
        }));
      }
      // Reject the pending push with NotSupportedError.
      if (this.pending_promise_func_) {
        this.pending_promise_func_(
            createNDEFError(device.mojom.NDEFErrorType.NOT_SUPPORTED));
        this.pending_promise_func_ = null;
      }
    }

    setIsFormattedTag(isFormatted) {
      this.is_formatted_tag_ = isFormatted;
    }

    simulateDataTransferFails() {
      this.data_transfer_failed_ = true;
    }

    simulateClosedPipe() {
      this.should_close_pipe_on_request_ = true;
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

    async initialize() {
      if (testInternal.initialized)
        throw new Error('Call reset() before initialize().');

      // Grant nfc permissions for Chromium testdriver.
      await test_driver.set_permission({ name: 'nfc' }, 'granted', false);

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
