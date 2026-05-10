// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.resolvedOptions
description: Verifies the branding check for the "resolvedOptions" function of the Segmenter prototype object.
info: |
    Intl.Segmenter.prototype.resolvedOptions ()

    2. If Type(pr) is not Object or pr does not have an [[InitializedSegmenter]] internal slot, throw a TypeError exception.
features: [Intl.Segmenter]
---*/

const resolvedOptions = Intl.Segmenter.prototype.resolvedOptions;

assert.sameValue(typeof resolvedOptions, "function");

assert.throws(TypeError, () => resolvedOptions.call(undefined), "undefined");
assert.throws(TypeError, () => resolvedOptions.call(null), "null");
assert.throws(TypeError, () => resolvedOptions.call(true), "true");
assert.throws(TypeError, () => resolvedOptions.call(""), "empty string");
assert.throws(TypeError, () => resolvedOptions.call(Symbol()), "symbol");
assert.throws(TypeError, () => resolvedOptions.call(1), "1");
assert.throws(TypeError, () => resolvedOptions.call({}), "plain object");
assert.throws(TypeError, () => resolvedOptions.call(Intl.Segmenter), "Intl.Segmenter");
assert.throws(TypeError, () => resolvedOptions.call(Intl.Segmenter.prototype), "Intl.Segmenter.prototype");
