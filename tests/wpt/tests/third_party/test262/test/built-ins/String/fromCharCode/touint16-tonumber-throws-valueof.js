// Copyright (C) 2026 Hyunjoon Kim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.fromcharcode
description: >
    String.fromCharCode propagates abrupt completion from ToNumber via ToUint16. (ToPrimitive/valueOf throws)
info: |
    String.fromCharCode ( ..._codeUnits_ )

    2. For each element _next_ of _codeUnits_, do
      a. Let _nextCU_ be the code unit whose numeric value is ℝ(? ToUint16(_next_)).

    ToUint16 ( _argument_ )

    1. Let _number_ be ? ToNumber(_argument_).

    ToNumber ( _argument_ )
    
    8. Let _primValue_ be ? ToPrimitive(_argument_, ~number~).    
---*/

assert.throws(Test262Error, function () {
  String.fromCharCode({
    valueOf: function () { throw new Test262Error(); }
  });
}, "ToNumber throws when its argument's ToPrimitive step calls a throwing valueOf, and String.fromCharCode must propagate it.");
