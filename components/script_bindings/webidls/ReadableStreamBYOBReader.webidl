/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://streams.spec.whatwg.org/#byob-reader-class-definition

[Exposed=*]
interface ReadableStreamBYOBReader {
  [Throws]
  constructor(ReadableStream stream);

  [NewObject]
  Promise<ReadableStreamReadResult> read(ArrayBufferView view,
      optional ReadableStreamBYOBReaderReadOptions options = {}
  );

  [Throws]
  undefined releaseLock();
};
ReadableStreamBYOBReader includes ReadableStreamGenericReader;

dictionary ReadableStreamBYOBReaderReadOptions {
  [EnforceRange] unsigned long long min = 1;
};


// The ByteTeeReadIntoRequest interface is entirely internal to Servo, and should not be accessible to
// web pages.
[LegacyNoInterfaceObject, Exposed=(Window,Worker)]
// Need to escape "ByteTeeReadIntoRequest" so it's treated as an identifier.
interface _ByteTeeReadIntoRequest {
};

// The ByteTeeReadRequest interface is entirely internal to Servo, and should not be accessible to
// web pages.
[LegacyNoInterfaceObject, Exposed=(Window,Worker)]
// Need to escape "ByteTeeReadRequest" so it's treated as an identifier.
interface _ByteTeeReadRequest {
};

// The ByteTeeUnderlyingSource interface is entirely internal to Servo, and should not be accessible to
// web pages.
[LegacyNoInterfaceObject, Exposed=(Window,Worker)]
// Need to escape "ByteTeeUnderlyingSource" so it's treated as an identifier.
interface _ByteTeeUnderlyingSource {
};
