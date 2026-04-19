// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: Behavior when `lastIndex` property of "this" value is non-writable
info: |
    [...]
    3. If Type(pattern) is Object and pattern has a [[RegExpMatcher]] internal
       slot, then
       a. If flags is not undefined, throw a TypeError exception.
       b. Let P be the value of pattern's [[OriginalSource]] internal slot.
       c. Let F be the value of pattern's [[OriginalFlags]] internal slot.
    4. Else,
       [...]
    5. Return ? RegExpInitialize(O, P, F).

    21.2.3.2.2 Runtime Semantics: RegExpInitialize

    [...]
    12. Perform ? Set(obj, "lastIndex", 0, true).
---*/

var subject = /initial/;
Object.defineProperty(subject, 'lastIndex', { value: 45, writable: false });

assert.throws(TypeError, function() {
  subject.compile(/updated/gi);
});

assert.sameValue(
  subject.toString(),
  new RegExp('updated', 'gi').toString(),
  '[[OriginalSource]] internal slot'
);
assert.sameValue(subject.lastIndex, 45);
