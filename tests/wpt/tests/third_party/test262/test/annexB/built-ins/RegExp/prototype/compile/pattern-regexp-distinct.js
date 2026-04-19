// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: RegExp is re-initialized when invoked with a distinct instance
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
---*/

var subject = /abc/gim;
var pattern = /def/;
var result;
subject.lastIndex = 23;
pattern.lastIndex = 45;

result = subject.compile(pattern);

assert.sameValue(result, subject, 'method return value');
assert.sameValue(subject.lastIndex, 0);
assert.sameValue(pattern.lastIndex, 45);

assert.sameValue(subject.toString(), new RegExp('def').toString());
assert.sameValue(
  subject.test('def'), true, '[[RegExpMatcher]] internal slot (source)'
);
assert.sameValue(
  subject.test('DEF'), false, '[[RegExpMatch]] internal slot (flags)'
);
