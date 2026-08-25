// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Type coercion of second argument
es6id: 21.2.5.8
info: |
    21.2.5.8 RegExp.prototype [ @@replace ] ( string, replaceValue )

    [...]
    6. Let functionalReplace be IsCallable(replaceValue).
    7. If functionalReplace is false, then
       a. Let replaceValue be ToString(replaceValue).
    [...]
features: [Symbol.replace]
---*/

var arg = {
  valueOf: function() {
    throw new Test262Error('This method should not be invoked.');
  },
  toString: function() {
    return 'toString value';
  }
};

assert.sameValue(/./[Symbol.replace]('string', arg), 'toString valuetring');
