// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks that Segmenter can be subclassed.
info: |
    Intl.Segmenter ( [ locales [ , options ] ] )

    2. Let segmenter be ? OrdinaryCreateFromConstructor(NewTarget, "%SegmenterPrototype%", « [[InitializedSegmenter]] »).
includes: [compareArray.js]
features: [Intl.Segmenter]
---*/

function segments(iterator) {
  return [...iterator].map(result => result.segment);
}

class CustomSegmenter extends Intl.Segmenter {
  constructor(locales, options) {
    super(locales, options);
    this.isCustom = true;
  }
}

const locale = "de";
const value = "Hello";

const real_segmenter = new Intl.Segmenter(locale);
assert.sameValue(real_segmenter.isCustom, undefined, "Custom property");

const custom_segmenter = new CustomSegmenter(locale);
assert.sameValue(custom_segmenter.isCustom, true, "Custom property");

assert.compareArray(segments(custom_segmenter.segment(value)),
                    segments(real_segmenter.segment(value)),
                    "Direct call");

assert.compareArray(segments(Intl.Segmenter.prototype.segment.call(custom_segmenter, value)),
                    segments(Intl.Segmenter.prototype.segment.call(real_segmenter, value)),
                    "Indirect call");

assert.sameValue(Object.getPrototypeOf(custom_segmenter), CustomSegmenter.prototype, "Prototype");
assert.sameValue(Object.getPrototypeOf(CustomSegmenter), Intl.Segmenter,
                 "Object.getPrototypeOf(CustomSegmenter) returns Intl.Segmenter");
assert.sameValue(Object.getPrototypeOf(CustomSegmenter.prototype), Intl.Segmenter.prototype,
                 "Object.getPrototypeOf(CustomSegmenter.prototype) returns Intl.Segmenter.prototype");
assert.sameValue(custom_segmenter instanceof Intl.Segmenter, true,
                 "The result of `custom_segmenter instanceof Intl.Segmenter` is true");

assert.throws(TypeError, function() {
  CustomSegmenter();
});
