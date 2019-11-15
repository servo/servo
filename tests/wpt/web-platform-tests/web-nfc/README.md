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
    setReadingMessage(NDEFMessageInit message); // Sets message that is used to deliver NFC reading updates.
    setPendingPushCompleted(boolean result); // Sets if the pending push is completed.
    pushedMessage(); // Gets the pushed `NDEFMessageSource`.
    pushOptions(); // Gets the pushed `NDEFPushOptions`.
    setIsNDEFTech(boolean isNDEF); // Sets if the NFC device exposes NDEF technology.
    setIsFormattedTag(boolean isFormatted); // Sets if the NFC tag has formatted NDEF message.
  };
```

The Chromium implementation of the `WebNFCTest` interface is located in
[nfc-mock.js](../resources/chromium/nfc-mock.js).

Other browser vendors should provide their own implementations of
the `WebNFCTest` interfaces.
