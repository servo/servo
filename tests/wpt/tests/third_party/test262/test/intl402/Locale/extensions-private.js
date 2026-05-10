// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale
description: >
    Verifies handling of options with privateuse tags.
info: |
    ApplyOptionsToTag( tag, options )

    
    ...
    9. If tag matches neither the privateuse nor the grandfathered production, then
    ...

features: [Intl.Locale]
---*/

assert.throws(RangeError, () => new Intl.Locale("x-default", {
  language: "fr",
  script: "Cyrl",
  region: "DE",
  numberingSystem: "latn",
}));
