// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 19.5.6.1.1
description: >
  A new instance has the message property if created with a parameter
info: |
  19.5.6.1.1 NativeError ( message )

  ...
  4. If message is not undefined, then
    a. Let msg be ToString(message).
    b. Let msgDesc be the PropertyDescriptor{[[Value]]: msg, [[Writable]]: true,
    [[Enumerable]]: false, [[Configurable]]: true}.
    c. Let status be DefinePropertyOrThrow(O, "message", msgDesc).
  ...
includes: [propertyHelper.js]

---*/

class Err extends ReferenceError {}

Err.prototype.message = 'custom-reference-error';

var err1 = new Err('foo 42');

verifyProperty(err1, 'message', {
  value: 'foo 42',
  writable: true,
  enumerable: false,
  configurable: true,
});

var err2 = new Err();
assert.sameValue(err2.hasOwnProperty('message'), false);
assert.sameValue(err2.message, 'custom-reference-error');
