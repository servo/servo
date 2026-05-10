// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: Behavior when provided pattern cannot be coerced to a string
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

    1. If pattern is undefined, let P be the empty String.
    2. Else, let P be ? ToString(pattern).
features: [Symbol]
---*/

var symbol = Symbol('');
var subject = /./;
var badToString = {
  toString: function() {
    throw new Test262Error();
  }
};
subject.lastIndex = 99;

assert.throws(Test262Error, function() {
  /./.compile(badToString);
});

assert.throws(TypeError, function() {
  /./.compile(symbol);
});

assert.sameValue(subject.lastIndex, 99);
