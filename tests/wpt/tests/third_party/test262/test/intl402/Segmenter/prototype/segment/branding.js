// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.segment
description: Verifies the branding check for the "segment" function of the Segmenter prototype object.
info: |
    Intl.Segmenter.prototype.segment( string )

    2. If Type(segment) is not Object or segment does not have an [[InitializedSegmenter]] internal slot, throw a TypeError exception.
features: [Intl.Segmenter]
---*/

const segment = Intl.Segmenter.prototype.segment;

assert.sameValue(typeof segment, "function");

assert.throws(TypeError, () => segment.call(undefined), "undefined");
assert.throws(TypeError, () => segment.call(null), "null");
assert.throws(TypeError, () => segment.call(true), "true");
assert.throws(TypeError, () => segment.call(""), "empty string");
assert.throws(TypeError, () => segment.call(Symbol()), "symbol");
assert.throws(TypeError, () => segment.call(1), "1");
assert.throws(TypeError, () => segment.call({}), "plain object");
assert.throws(TypeError, () => segment.call(Intl.Segmenter), "Intl.Segmenter");
assert.throws(TypeError, () => segment.call(Intl.Segmenter.prototype), "Intl.Segmenter.prototype");
