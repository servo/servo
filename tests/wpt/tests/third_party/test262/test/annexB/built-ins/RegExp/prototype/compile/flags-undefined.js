// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: Behavior when flags is undefined
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

    [...]
    3. If flags is undefined, let F be the empty String.
    [...]
---*/

var subject, result;

subject = /abc/ig;

result = subject.compile('def');

assert.sameValue(result, subject, 'method return value (unspecified)');
assert.sameValue(
  subject.flags, new RegExp('def').flags, '[[OriginalFlags]] (unspecified)'
);
assert.sameValue(
  subject.test('DEF'), false, '[[RegExpMatcher]] internal slot (unspecified)'
);

subject = /abc/gi;

result = subject.compile('def', undefined);

assert.sameValue(result, subject, 'method return value (explicit undefined)');
assert.sameValue(
  subject.flags,
  new RegExp('def').flags,
  '[[OriginalSource]] (explicit undefined)'
);
assert.sameValue(
  subject.test('DEF'),
  false,
  '[[RegExpMatcher]] internal slot (explicit undefined)'
);
