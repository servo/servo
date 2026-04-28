// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.frombase64
description: Uint8Array.fromBase64 triggers effects of the "alphabet" and "lastChunkHandling" getters, but does not perform toString on the results
includes: [compareArray.js]
features: [uint8array-base64, TypedArray]
---*/

assert.throws(TypeError, function() {
  Uint8Array.fromBase64("Zg==", { alphabet: Object("base64") });
});

assert.throws(TypeError, function() {
  Uint8Array.fromBase64("Zg==", { lastChunkHandling: Object("loose") });
});


var toStringCalls = 0;
var throwyToString = {
  toString: function() {
    toStringCalls += 1;
    throw new Test262Error("toString called");
  }
};
assert.throws(TypeError, function() {
  Uint8Array.fromBase64("Zg==", { alphabet: throwyToString });
});
assert.sameValue(toStringCalls, 0);

assert.throws(TypeError, function() {
  Uint8Array.fromBase64("Zg==", { lastChunkHandling: throwyToString });
});
assert.sameValue(toStringCalls, 0);


var alphabetAccesses = 0;
var base64UrlOptions = {};
Object.defineProperty(base64UrlOptions, "alphabet", {
  get: function() {
    alphabetAccesses += 1;
    return "base64url";
  }
});
var arr = Uint8Array.fromBase64("x-_y", base64UrlOptions);
assert.compareArray(arr, [199, 239, 242]);
assert.sameValue(alphabetAccesses, 1);

var lastChunkHandlingAccesses = 0;
var strictOptions = {};
Object.defineProperty(strictOptions, "lastChunkHandling", {
  get: function() {
    lastChunkHandlingAccesses += 1;
    return "strict";
  }
});
var arr = Uint8Array.fromBase64("Zg==", strictOptions);
assert.compareArray(arr, [102]);
assert.sameValue(lastChunkHandlingAccesses, 1);
