// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Intl.Segmenter instance object is created from %SegmenterPrototype%.
info: |
    Intl.Segmenter ([ locales [ , options ]])

    2. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%SegmenterPrototype%", « [[InitializedSegmenter]] »).
features: [Intl.Segmenter]
---*/

const value = new Intl.Segmenter();
assert.sameValue(
  Object.getPrototypeOf(value),
  Intl.Segmenter.prototype,
  "Object.getPrototypeOf(value) equals the value of Intl.Segmenter.prototype"
);
