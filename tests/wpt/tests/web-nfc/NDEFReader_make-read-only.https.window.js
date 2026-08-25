// META: script=resources/nfc-helpers.js

// NDEFReader.makeReadOnly method
// https://w3c.github.io/web-nfc/#dom-ndefreader-makereadonly

'use strict';

const invalid_signals = ['string', 123, {}, true, Symbol(), () => {}, self];

nfc_test(async t => {
  await test_driver.set_permission({name: 'nfc'}, 'denied');
  const ndef = new NDEFReader();
  await promise_rejects_dom(t, 'NotAllowedError', ndef.makeReadOnly());
}, 'NDEFReader.makeReadOnly should fail if user permission is not granted.');

// We do not provide NFC mock here to simulate that there has no available
// implementation for NFC Mojo interface.
nfc_test(async (t, mockNFC) => {
  mockNFC.simulateClosedPipe();
  const ndef = new NDEFReader();
  await promise_rejects_dom(t, 'NotSupportedError', ndef.makeReadOnly());
}, 'NDEFReader.makeReadOnly should fail if no implementation for NFC Mojo interface is available.');

nfc_test(async (t, mockNFC) => {
  const ndef = new NDEFReader();
  const controller = new AbortController();

  // Make sure makeReadOnly is pending
  mockNFC.setPendingMakeReadOnlyCompleted(false);
  const p = ndef.makeReadOnly({signal: controller.signal});
  const rejected = promise_rejects_dom(t, 'AbortError', p);
  let callback_called = false;
  await new Promise(resolve => {
    t.step_timeout(() => {
      callback_called = true;
      controller.abort();
      resolve();
    }, 10);
  });
  await rejected;
  assert_true(callback_called, 'timeout should have caused the abort');
}, 'NDEFReader.makeReadOnly should fail if request is aborted before makeReadOnly happends.');

nfc_test(async t => {
  const ndef = new NDEFReader();
  const controller = new AbortController();
  assert_false(controller.signal.aborted);
  controller.abort();
  assert_true(controller.signal.aborted);
  await promise_rejects_dom(
      t, 'AbortError', ndef.makeReadOnly({signal: controller.signal}));
}, 'NDEFReader.makeReadOnly should fail if signal is already aborted.');

nfc_test(async t => {
  const ndef = new NDEFReader();
  const promises = [];
  invalid_signals.forEach(invalid_signal => {
    promises.push(promise_rejects_js(
        t, TypeError, ndef.makeReadOnly({signal: invalid_signal})));
  });
  await Promise.all(promises);
}, 'NDEFReader.write should fail if signal is not an AbortSignal.');

nfc_test(async (t, mockNFC) => {
  const ndef1 = new NDEFReader();
  const ndef2 = new NDEFReader();
  const controller = new AbortController();
  const p1 = ndef1.makeReadOnly({signal: controller.signal});

  // Even though makeReadOnly request is grantable,
  // this abort should be processed synchronously.
  controller.abort();
  await promise_rejects_dom(t, 'AbortError', p1);

  await ndef2.makeReadOnly();
}, 'Synchronously signaled abort.');

nfc_test(async (t, mockNFC) => {
  const ndef = new NDEFReader();
  mockNFC.setHWStatus(NFCHWStatus.DISABLED);
  await promise_rejects_dom(t, 'NotReadableError', ndef.makeReadOnly());
}, 'NDEFReader.makeReadOnly should fail when NFC HW is disabled.');

nfc_test(async (t, mockNFC) => {
  const ndef = new NDEFReader();
  mockNFC.setHWStatus(NFCHWStatus.NOT_SUPPORTED);
  await promise_rejects_dom(t, 'NotSupportedError', ndef.makeReadOnly());
}, 'NDEFReader.makeReadOnly should fail when NFC HW is not supported.');

nfc_test(async () => {
  await new Promise((resolve, reject) => {
    const iframe = document.createElement('iframe');
    iframe.srcdoc = `<script>
                      window.onmessage = async (message) => {
                        if (message.data === "Ready") {
                          try {
                            const ndef = new NDEFReader();
                            await ndef.makeReadOnly();
                            parent.postMessage("Failure", "*");
                          } catch (error) {
                            if (error.name == "InvalidStateError") {
                              parent.postMessage("Success", "*");
                            } else {
                              parent.postMessage("Failure", "*");
                            }
                          }
                        }
                      };
                    </script>`;
    iframe.onload = () => iframe.contentWindow.postMessage('Ready', '*');
    document.body.appendChild(iframe);
    window.onmessage = message => {
      if (message.data == 'Success') {
        resolve();
      } else if (message.data == 'Failure') {
        reject();
      }
    }
  });
}, 'Test that WebNFC API is not accessible from iframe context.');

nfc_test(async () => {
  const ndef = new NDEFReader();
  await ndef.makeReadOnly();
}, 'NDEFReader.makeReadOnly should succeed when NFC HW is enabled');

nfc_test(async (t, mockNFC) => {
  const ndef1 = new NDEFReader();
  const ndef2 = new NDEFReader();

  // Make sure the first makeReadOnly will be pending.
  mockNFC.setPendingMakeReadOnlyCompleted(false);

  const p1 = ndef1.makeReadOnly();
  const p2 = ndef2.makeReadOnly();

  await promise_rejects_dom(t, 'AbortError', p1);
  await p2;
}, 'NDEFReader.makeReadOnly should replace all previously configured makeReadOnly operations.');

nfc_test(async () => {
  const ndef = new NDEFReader();

  const controller1 = new AbortController();
  await ndef.makeReadOnly({signal: controller1.signal});

  const controller2 = new AbortController();
  const promise = ndef.makeReadOnly({signal: controller2.signal});
  controller1.abort();
  await promise;
}, 'NDEFReader.makeReadOnly signals are independent.');

nfc_test(async (t, mockNFC) => {
  // Make sure the makeReadOnly will be pending in the mock.
  mockNFC.setPendingMakeReadOnlyCompleted(false);

  const ndef1 = new NDEFReader();
  const promise = ndef1.makeReadOnly();

  // Just to make sure the makeReadOnly() request has already reached to the
  // mock.
  const ndef2 = new NDEFReader();
  await ndef2.scan();

  mockNFC.simulateNonNDEFTagDiscovered();
  await promise_rejects_dom(t, 'NotSupportedError', promise);
}, 'NDEFReader.makeReadOnly should fail when the NFC device coming up does not expose \
NDEF technology.');

nfc_test(async (t, mockNFC) => {
  const ndef = new NDEFReader();
  mockNFC.simulateDataTransferFails();
  await promise_rejects_dom(t, 'NetworkError', ndef.makeReadOnly());
}, 'NDEFReader.makeReadOnly should fail with NetworkError when NFC data transfer fails.');
