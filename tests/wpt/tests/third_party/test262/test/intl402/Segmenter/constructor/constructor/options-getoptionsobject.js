// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter
description: Checks handling of non-object option arguments to the Segmenter constructor.
info: |
    Intl.Segmenter ([ locales [ , options ]])

features: [Intl.Segmenter,BigInt]
---*/

const optionsArguments = [
  null,
  true,
  false,
  "test",
  7,
  Symbol(),
  123456789n,
];

for (const options of optionsArguments) {
  assert.throws(TypeError, function() { new Intl.Segment([], options) })
}
