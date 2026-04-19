// Copyright (C) 2017 Daniel Ehrenberg. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializecollator
description: >
  Monkey-patching Object.prototype does not change the default
  options for Collator as a null prototype is used.
info: |
  InitializeCollator ( collator, locales, options )

    1. If _options_ is *undefined*, then
      1. Let _options_ be ObjectCreate(*null*).
---*/

if (new Intl.Collator("en").resolvedOptions().locale === "en") {
  Object.prototype.sensitivity = "base";
  assert.sameValue("a".localeCompare("A"), -1);
}
