// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: Uint8Array.fromBase64 throws if its argument is not a string
features: [uint8array-base64, TypedArray]
---*/

var toStringCalls = 0;
var throwyToString = {
  toString: function() {
    toStringCalls += 1;
    throw new Test262Error("toString called");
  }
};

assert.throws(TypeError, function() {
  Uint8Array.fromBase64(throwyToString);
});
assert.sameValue(toStringCalls, 0);


var optionAccesses = 0;
var touchyOptions = {};
Object.defineProperty(touchyOptions, "alphabet", {
  get: function() {
    optionAccesses += 1;
    throw new Test262Error("alphabet accessed");
  }
});
Object.defineProperty(touchyOptions, "lastChunkHandling", {
  get: function() {
    optionAccesses += 1;
    throw new Test262Error("lastChunkHandling accessed");
  }
});
assert.throws(TypeError, function() {
  Uint8Array.fromBase64(throwyToString, touchyOptions);
});
assert.sameValue(toStringCalls, 0);
assert.sameValue(optionAccesses, 0);
