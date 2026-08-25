// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: Behavior when pattern is undefined
info: |
    [...]
    3. If Type(pattern) is Object and pattern has a [[RegExpMatcher]] internal
       slot, then
       [...]
    4. Else,
       a. Let P be pattern.
       b. Let F be flags.
    5. Return ? RegExpInitialize(O, P, F).

    21.2.3.2.2 Runtime Semantics: RegExpInitialize

    1. If pattern is undefined, let P be the empty String.
    [...]
---*/

var subject;

subject = /abc/;
assert.sameValue(
  subject.compile(), subject, 'method return value (unspecified)'
);
assert.sameValue(
  subject.source, new RegExp('').source, '[[OriginalSource]] (unspecified)'
);
assert.sameValue(
  subject.test(''), true, '[[RegExpMatcher]] internal slot (unspecified)'
);

subject = /abc/;
assert.sameValue(
  subject.compile(undefined),
  subject,
  'method return value (explicit undefined)'
);
assert.sameValue(
  subject.source,
  new RegExp('').source,
  '[[OriginalSource]] (explicit undefined)'
);
assert.sameValue(
  subject.test(''),
  true,
  '[[RegExpMatcher]] internal slot (explicit undefined)'
);
