// Copyright (C) 2026 Garham Lee. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.flatmap
description: >
    Array.prototype.flatMap throws when _soucreLen_ is not 0 and _mapperFunction_ throws.
info: |
    Array.prototype.flatMap ( _mapperFunction_ [ , _thisArg_ ] )

    1. Let _O_ be ? ToObject(*this* value).
    2. Let _sourceLen_ be ? LengthOfArrayLike(_O_).
    5. Perform ? FlattenIntoArray(_A_, _O_, _sourceLen_, 0, 1, _mapperFunction_, _thisArg_).

    FlattenIntoArray (_target_,_source_,_sourceLen_,_start_,_depth_,optional _mapperFunction_,optional _thisArg_,)

    2. Let _targetIndex_ be _start_.
    3. Let _sourceIndex_ be *+0*<sub>𝔽</sub>.
    4. Repeat, while ℝ(_sourceIndex_) &lt; _sourceLen_,
        a. Let _P_ be ! ToString(_sourceIndex_).
        b. Let _exists_ be ? HasProperty(_source_, _P_).
        c. If _exists_ is *true*, then
            i. Let _element_ be ? Get(_source_, _P_).
            ii. If _mapperFunction_ is present, then
                1. Set _element_ to ? Call(_mapperFunction_, _thisArg_, « _element_, _sourceIndex_, _source_ »).
features: [Array.prototype.flatMap]
---*/
//Check #1
assert.throws(Test262Error, function () {
    [0].flatMap(x => {throw new Test262Error});
});

//Check #2
var callcount = 0;
[].flatMap(() => {
    callcount += 1;
    throw new Test262Error;
});

assert.sameValue(callcount, 0, "If _soucreLen_ is 0, _mapperFunction_ should not be called.")
