// Copyright (C) 2018 Peter Wong. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: pending
description: Re-throws errors thrown while accessing RegExp's @@match property
info: |
  RegExp.prototype [ @@matchAll ] ( string )
    [...]
    3. Return ? MatchAllIterator(R, string).

  MatchAllIterator ( R, O )
    [...]
    2. If ? IsRegExp(R) is true, then
      [...]
features: [Symbol.match, Symbol.matchAll]
---*/

var obj = {
  get [Symbol.match]() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.matchAll].call(obj, '');
});
