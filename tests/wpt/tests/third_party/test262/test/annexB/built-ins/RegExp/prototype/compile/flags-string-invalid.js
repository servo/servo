// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: Behavior when flags is a string describing an invalid flag set
info: |
    [...]
    5. Return ? RegExpInitialize(O, P, F).

    21.2.3.2.2 Runtime Semantics: RegExpInitialize

    [...]
    3. If flags is undefined, let F be the empty String.
    4. Else, let F be ? ToString(flags).
    5. If F contains any code unit other than "g", "i", "m", "u", or "y" or if
       it contains the same code unit more than once, throw a SyntaxError
       exception.
---*/

var subject = /abcd/ig;

assert.throws(SyntaxError, function() {
  subject.compile('', 'igi');
}, 'invalid flags: igi');

assert.throws(SyntaxError, function() {
  subject.compile('', 'gI');
}, 'invalid flags: gI');

assert.throws(SyntaxError, function() {
  subject.compile('', 'w');
}, 'invalid flags: w');

assert.sameValue(
  subject.toString(),
  new RegExp('abcd', 'ig').toString(),
  '[[OriginalSource]] internal slot'
);

assert.sameValue(
  subject.test('AbCD'), true, '[[RegExpMatcher]] internal slot'
);
