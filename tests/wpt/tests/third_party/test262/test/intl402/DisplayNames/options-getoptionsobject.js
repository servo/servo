// Copyright 2021 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DisplayNames
description: Checks handling of non-object option arguments to the DisplayNames constructor.
info: |
    Intl.DisplayNames ( locales, options )
features: [Intl.DisplayNames,BigInt]
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
  assert.throws(TypeError, function() { new Intl.DisplayNames([], options) })
}
