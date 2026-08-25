// Copyright 2024 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: >
  Checks that the special handling for number of fractional digits when formatting currency values in "standard" notation is ignored when using "compact", "engineering", or "scientific" notation.
info: |
  Intl.DateTimeFormat ( [ locales [ , options ] ] )

  ...
  19. If style is "currency" and "notation" is "standard", then
  ...
  20. Else,
    a. mnfdDefault be 0.
    b. If style is "percent", then
      i. Let mxfdDefault be 0.
    c. Else,
      i. Let mxfdDefault be 3.
---*/

for (const notation of ["compact", "engineering", "scientific"]) {
  for (const currency of ["JPY", "KWD", "USD"]) {
    let nf = new Intl.NumberFormat('en-US', {style: "currency", currency, notation});
    const resolvedOptions = nf.resolvedOptions();
    const minimumFractionDigits = resolvedOptions.minimumFractionDigits;
    const maximumFractionDigits = resolvedOptions.maximumFractionDigits;

    assert.sameValue(minimumFractionDigits, 0, "Didn't get correct minimumFractionDigits for " + currency + " in " + notation + " notation.");
    assert.sameValue(maximumFractionDigits, notation !== "compact" ? 3 : 0, "Didn't get correct maximumFractionDigits for " + currency + " in " + notation + " notation.");
  }
}


