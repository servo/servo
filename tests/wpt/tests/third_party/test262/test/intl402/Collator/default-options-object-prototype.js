// Copyright (C) 2017 Daniel Ehrenberg. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.collator
description: >
  Monkey-patching Object.prototype does not change the default
  options for Collator as a null prototype is used.
---*/

let defaultSensitivity = new Intl.Collator("en").resolvedOptions().sensitivity;

Object.prototype.sensitivity = "base";
let collator = new Intl.Collator("en");
assert.sameValue(collator.resolvedOptions().sensitivity, defaultSensitivity);
