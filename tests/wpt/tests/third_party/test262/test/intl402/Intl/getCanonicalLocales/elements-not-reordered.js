// Copyright (C) 2017 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.getcanonicallocales
description: >
  Language tags are not reordered.
info: |
  8.2.1 Intl.getCanonicalLocales (locales)
    1. Let ll be ? CanonicalizeLocaleList(locales).
    2. Return CreateArrayFromList(ll).

  9.2.1 CanonicalizeLocaleList (locales)
    ...
    7. Repeat, while k < len
      ...
      c. If kPresent is true, then
        ...
        vi. If canonicalizedTag is not an element of seen, append canonicalizedTag as the last element of seen.
        ...
---*/

var canonicalLocales = Intl.getCanonicalLocales(["zu", "af"]);

assert.sameValue(canonicalLocales.length, 2);
assert.sameValue(canonicalLocales[0], "zu");
assert.sameValue(canonicalLocales[1], "af");
