// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Behavior with a custom RegExp exec
info: |
  %RegExpStringIteratorPrototype%.next ( )
    [...]
    9. Let match be ? RegExpExec(R, S).

  Runtime Semantics: RegExpExec ( R, S )
    1. Assert: Type(R) is Object.
    2. Assert: Type(S) is String.
    3. Let exec be ? Get(R, "exec").
    4. If IsCallable(exec) is true, then
      [...]
    5. If R does not have a [[RegExpMatcher]] internal slot, throw a
       TypeError exception.
    6. Return ? RegExpBuiltinExec(R, S).
features: [Symbol.matchAll]
includes: [compareArray.js, compareIterator.js, regExpUtils.js]
---*/

function TestWithRegExpExec(exec) {
  RegExp.prototype.exec = exec;

  var regexp = /\w/g;
  var str = 'a*b';

  assert.compareIterator(regexp[Symbol.matchAll](str), [
    matchValidator(['a'], 0, str),
    matchValidator(['b'], 2, str)
  ]);
}

TestWithRegExpExec(undefined);
TestWithRegExpExec(null);
TestWithRegExpExec(5);
TestWithRegExpExec(true);
TestWithRegExpExec(Symbol());
