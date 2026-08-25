// Copyright (C) 2019 ta7sudan. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-proxy-object-internal-methods-and-internal-slots-getprototypeof
description: >
  instanceof operator will return true if trap result is the prototype of the function.
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
features: [Proxy]
---*/

function Custom() {}

var p = new Proxy({}, {
  getPrototypeOf() {
    return Custom.prototype;
  }
});

assert(p instanceof Custom);
