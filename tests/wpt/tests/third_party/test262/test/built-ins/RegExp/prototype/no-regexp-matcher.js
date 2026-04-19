// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-properties-of-the-regexp-prototype-object
description: >
  The RegExp prototype object does not have a [[RegExpMatcher]] internal slot
info: |
  The RegExp prototype object is an ordinary object. It is not a RegExp
  instance and does not have a [[RegExpMatcher]] internal slot or any of the
  other internal slots of RegExp instance objects.

  21.2.5.2 RegExp.prototype.exec

  1. Let R be the this value.
  2. If Type(R) is not Object, throw a TypeError exception.
  3. If R does not have a [[RegExpMatcher]] internal slot, throw a TypeError
     exception.
---*/

assert.throws(TypeError, function() {
  RegExp.prototype.exec('');
});
