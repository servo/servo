// Copyright (C) 2019 Sony Interactive Entertainment Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-optional-chaining-chain-evaluation
description: >
  optional call must preserve this context, as with a non-optional call
info: |
  OptionalChain : ?. Arguments
    1. Let thisChain be this OptionalChain.
    2. Let tailCall be IsInTailPosition(thisChain).
    3. Return ? EvaluateCall(baseValue, baseReference, Arguments, tailCall).
features: [optional-chaining]
---*/

const a = {
    b() { return this._b; },
    _b: { c: 42 }
};

assert.sameValue(a?.b().c, 42);
assert.sameValue((a?.b)().c, 42);

assert.sameValue(a.b?.().c, 42);
assert.sameValue((a.b)?.().c, 42);

assert.sameValue(a?.b?.().c, 42);
assert.sameValue((a?.b)?.().c, 42);
