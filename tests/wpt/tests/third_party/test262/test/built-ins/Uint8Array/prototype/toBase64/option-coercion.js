// Copyright (C) 2024 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-uint8array.prototype.tobase64
description: Uint8Array.prototype.toBase64 triggers effects of the "alphabet" getter, but does not perform toString on the result
features: [uint8array-base64, TypedArray]
---*/

assert.throws(TypeError, function() {
  (new Uint8Array(2)).toBase64({ alphabet: Object("base64") });
});


var toStringCalls = 0;
var throwyToString = {
  toString: function() {
    toStringCalls += 1;
    throw new Test262Error("toString called on alphabet value");
  }
};
assert.throws(TypeError, function() {
  (new Uint8Array(2)).toBase64({ alphabet: throwyToString });
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
assert.sameValue((new Uint8Array([199, 239, 242])).toBase64(base64UrlOptions), "x-_y");
assert.sameValue(alphabetAccesses, 1);

// side-effects from the getter on the receiver are reflected in the result
var array = new Uint8Array([0]);
var receiverMutatingOptions = {};
Object.defineProperty(receiverMutatingOptions, "alphabet", {
  get: function() {
    array[0] = 255;
    return "base64";
  }
});
var result = array.toBase64(receiverMutatingOptions);
assert.sameValue(result, "/w==");
assert.sameValue(array[0], 255);
