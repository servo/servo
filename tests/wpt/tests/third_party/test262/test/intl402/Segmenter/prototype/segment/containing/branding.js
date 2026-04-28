// Copyright 2020 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%segmentsprototype%.containing
description: Verifies the branding check for the "segment" function of the %Segments.prototype%.containing.
info: |
    %Segments.prototype%.containing ( index )
    1. Let segments be the this value.
    2. Perform ? RequireInternalSlot(segments, [[SegmentsSegmenter]]).

features: [Intl.Segmenter]
---*/
const segment = (new Intl.Segmenter()).segment("123");
const containing = segment.containing;
assert.sameValue(typeof containing, "function");
assert.throws(TypeError, () => containing.call(undefined), "undefined");
assert.throws(TypeError, () => containing.call(null), "null");
assert.throws(TypeError, () => containing.call(true), "true");
assert.throws(TypeError, () => containing.call(""), "empty string");
assert.throws(TypeError, () => containing.call(Symbol()), "symbol");
assert.throws(TypeError, () => containing.call(1), "1");
assert.throws(TypeError, () => containing.call({}), "plain object");
assert.throws(TypeError, () => containing.call(Intl.Segmenter), "Intl.Segmenter");
assert.throws(TypeError, () => containing.call(Intl.Segmenter.prototype), "Intl.Segmenter.prototype");
assert.sameValue(undefined, containing.call(segment, -1));
