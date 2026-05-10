// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-ecmascript-standard-built-in-objects
description: >
  BigInt is a constructor, and does implement [[Construct]], but is not new target
info: |
  sec-bigint-constructor

  - is not intended to be used with the new operator or to be subclassed. It may be used as the value of an extends clause of a class definition but a super call to the BigInt constructor will cause an exception.

  sec-bigint-constructor-number-value

  1. If NewTarget is not undefined, throw a TypeError exception.
includes: [isConstructor.js]
features: [BigInt, Reflect.construct, arrow-function]
---*/

assert.sameValue(
  isConstructor(BigInt),
  true,
  'isConstructor(BigInt) must return true'
);

assert.throws(TypeError, () => {
  new BigInt({
    valueOf() {
      new Test262Error();
    }
  });
}, '`new BigInt({ valueOf() {new Test262Error();}})` throws TypeError');

