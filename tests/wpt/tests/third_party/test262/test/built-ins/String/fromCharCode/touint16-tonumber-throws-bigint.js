// Copyright (C) 2026 Hyunjoon Kim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.fromcharcode
description: >
    String.fromCharCode propagates abrupt completion from ToNumber when argument is BigInt via ToUint16.
info: |
    String.fromCharCode ( ..._codeUnits_ )

    2. For each element _next_ of _codeUnits_, do
      a. Let _nextCU_ be the code unit whose numeric value is ℝ(? ToUint16(_next_)).

    ToUint16 ( _argument_ )

    1. Let _number_ be ? ToNumber(_argument_).

    ToNumber ( _argument_ )
    
    2. If _argument_ is either a Symbol or a BigInt, throw a *TypeError* exception.
features: [BigInt]
---*/

assert.throws(TypeError, function () {
  String.fromCharCode(0n);
}, "ToNumber throws when argument is BigInt, and String.fromCharCode must propagate it.");
