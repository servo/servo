// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors thrown while accessing RegExp's exec property
info: |
  %RegExpStringIteratorPrototype%.next ( )
    [...]
    9. Let match be ? RegExpExec(R, S).

  Runtime Semantics: RegExpExec ( R, S )
    1. Assert: Type(R) is Object.
    2. Assert: Type(S) is String.
    3. Let exec be ? Get(R, "exec").
features: [Symbol.matchAll]
---*/

var iter = /./[Symbol.matchAll]('');

Object.defineProperty(RegExp.prototype, 'exec', {
  get() {
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  iter.next();
});
