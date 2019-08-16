The `nfc-helpers.js` requires an implementation of
the `WebNFCTest` interfaces, which should emulate platform Web NFC backends.

The `WebNFCTest` interface is defined as:

```
  class NFCTestChromium {
    initialize();  // Sets up the testing environment.
    async reset(); // Frees the resources.
    getMockNFC(); // Returns `MockNFC` interface.
  };

  class MockNFC {
    setHWStatus(number status); // Sets the hardware status.
    setCompatibility(NDEFCompatibility compatibility); // Sets NDEF accepted compatible devices
    setReadingMessage(NDEFMessageInit message, NDEFCompatibility compatibility); // Sets message that is used to deliver NFC reading updates with a specific NDEFCompatibility.
    setPendingPushCompleted(boolean result); // Sets if the pending push is completed.
    setPushShouldTimeout(boolean result); // Sets flag to trigger the pending push to timeout.
    pushedMessage(); // Gets the pushed `NDEFMessageSource`.
    pushOptions(); // Gets the pushed `NFCPushOptions`.
  };
```

The Chromium implementation of the `WebNFCTest` interface is located in
[nfc-mock.js](../resources/chromium/nfc-mock.js).

Other browser vendors should provide their own implementations of
the `WebNFCTest` interfaces.
