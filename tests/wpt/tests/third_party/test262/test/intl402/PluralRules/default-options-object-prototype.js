// Copyright (C) 2017 Igalia, S. L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializepluralrules
description: >
  Monkey-patching Object.prototype does not change the default
  options for PluralRules as a null prototype is used.
info: |
  InitializePluralRules ( collator, locales, options )

    1. If _options_ is *undefined*, then
      1. Let _options_ be ObjectCreate(*null*).
---*/

Object.prototype.type = "ordinal";
Object.prototype.notation = "compact";
let pluralRules = new Intl.PluralRules("en");
assert.sameValue(pluralRules.resolvedOptions().type, "cardinal");
assert.sameValue(pluralRules.resolvedOptions().notation, "standard");
