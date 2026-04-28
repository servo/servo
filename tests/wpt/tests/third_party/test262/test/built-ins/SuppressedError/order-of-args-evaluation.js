// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-suppressederror-constructor
description: >
  Process arguments in superclass-then-subclass order
info: |
  SuppressedError ( error, suppressed, message )

  3. If message is not undefined, then
    a. Let messageString be ? ToString(message).
    b. Perform CreateNonEnumerableDataPropertyOrThrow(O, "message", messageString).
  4. Perform CreateNonEnumerableDataPropertyOrThrow(O, "error", error).
  5. Perform CreateNonEnumerableDataPropertyOrThrow(O, "suppressed", suppressed).

features: [explicit-resource-management, Symbol.iterator]
---*/

let messageStringified = false;
const message = {
  toString() {
    messageStringified = true;
    return '';
  }
};
const error = {};
const suppressed = {};

const e = new SuppressedError(error, suppressed, message);

assert.sameValue(messageStringified, true);
const keys = Object.getOwnPropertyNames(e);

// Allow implementation-defined properties before "message" and after "suppressed".

const messageIndex = keys.indexOf("message");
assert.notSameValue(messageIndex, -1, "Expected 'message' to be defined");

const errorIndex = keys.indexOf("error");
assert.sameValue(errorIndex, messageIndex + 1, "Expected 'error' to be defined after 'message'");

const suppressedIndex = keys.indexOf("suppressed");
assert.sameValue(suppressedIndex, errorIndex + 1, "Expected 'suppressed' to be defined after 'error'");
