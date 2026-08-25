// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: Behavior when flags is a string describing a valid flag set
info: |
    [...]
    5. Return ? RegExpInitialize(O, P, F).

    21.2.3.2.2 Runtime Semantics: RegExpInitialize

    [...]
    3. If flags is undefined, let F be the empty String.
    4. Else, let F be ? ToString(flags).
    [...]
---*/

var subject = /a/g;

subject.compile('a', 'i');

assert.sameValue(
  subject.flags,
  new RegExp('a', 'i').flags,
  '[[OriginalFlags]] internal slot'
);
assert.sameValue(
  subject.test('A'),
  true,
  '[[RegExpMatcher]] internal slot (addition of `i` flag)'
);

subject.lastIndex = 1;
assert.sameValue(
  subject.test('A'),
  true,
  '[[RegExpMatcher]] internal slot (removal of `g` flag)'
);
