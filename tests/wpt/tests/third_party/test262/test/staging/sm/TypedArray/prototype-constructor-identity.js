// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  Typed array prototypes/constructors should be largely empty, inheriting most functionality from %TypedArray% and %TypedArray%.prototype
info: bugzilla.mozilla.org/show_bug.cgi?id=896116
esid: pending
---*/

/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/publicdomain/zero/1.0/
 */

var TypedArray = Object.getPrototypeOf(Int8Array);

assert.sameValue(TypedArray !== Function.prototype, true,
         "%TypedArray% should be in constructors' [[Prototype]] chains");
assert.sameValue(Object.getPrototypeOf(TypedArray), Function.prototype,
         "%TypedArray%.prototype should inherit from Function.prototype");

assert.sameValue(TypedArray.prototype.constructor, TypedArray,
         "bad %TypedArray%.prototype.constructor");

// Check a few different functions we implement are here, as a sanity check.
var typedArrayProps = Object.getOwnPropertyNames(TypedArray.prototype);
assert.sameValue(typedArrayProps.indexOf("copyWithin") >= 0, true);
assert.sameValue(typedArrayProps.indexOf("subarray") >= 0, true);
assert.sameValue(typedArrayProps.indexOf("set") >= 0, true);

anyTypedArrayConstructors.forEach(function(ctor) {
  assert.sameValue(Object.getPrototypeOf(ctor), TypedArray);

  var proto = ctor.prototype;

  // Inherited functions aren't present.
  var props = Object.getOwnPropertyNames(proto).sort();
  assert.sameValue(props[0], "BYTES_PER_ELEMENT");
  assert.sameValue(props[1], "constructor");
  if (ctor === Uint8Array) {
    assert.sameValue(props.length, 6);
  } else {
    assert.sameValue(props.length, 2);
  }

  // The inheritance chain should be set up properly.
  assert.sameValue(Object.getPrototypeOf(proto), TypedArray.prototype,
           "prototype should inherit from %TypedArray%.prototype");
});
