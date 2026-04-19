// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-getprototypeof
description: >
  instanceof operator observes the TypeError from a custom trap result that would return true if
  the target were extensible.
info: |
  Runtime Semantics: InstanceofOperator ( V, target )

  5. Return ? OrdinaryHasInstance(target, V).

  OrdinaryHasInstance ( C, O )

  4. Let P be ? Get(C, "prototype").
  ...
  6. Repeat,
    a. Set O to ? O.[[GetPrototypeOf]]().
    b. If O is null, return false.
    c. If SameValue(P, O) is true, return true.

  [[GetPrototypeOf]] ( )

  7. Let handlerProto be ? Call(trap, handler, « target »).
  8. If Type(handlerProto) is neither Object nor Null, throw a TypeError exception.
  9. Let extensibleTarget be ? IsExtensible(target).
  10. If extensibleTarget is true, return handlerProto.
  11. Let targetProto be ? target.[[GetPrototypeOf]]().
  12. If SameValue(handlerProto, targetProto) is false, throw a TypeError exception.
features: [Proxy]
---*/

function Custom() {}

var target = {};

var p = new Proxy(target, {
  getPrototypeOf() {
    return Custom.prototype;
  }
});

Object.preventExtensions(target);

assert.throws(TypeError, () => {
  p instanceof Custom
});
