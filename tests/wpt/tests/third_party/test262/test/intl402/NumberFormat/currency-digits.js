// Copyright 2024 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: >
    Tests that data for the number of fractional digits when formatting currency is used.
info: |
    15.1.1 Intl.NumberFormat ([ locales [ , options ]])

    19. If style is "currency" and "notation" is "standard", then
      a. Let currency be numberFormat.[[Currency]].
      b. Let cDigits be CurrencyDigits(currency).
      c. Let mnfdDefault be cDigits.
      d. Let mxfdDefault be cDigits.
author: Ben Allen
---*/

const nf = Intl.NumberFormat([], {style: "currency", currency: "USD"});
const max = nf.resolvedOptions().maximumFractionDigits;
const min = nf.resolvedOptions().minimumFractionDigits;

assert.sameValue(min, max, "Currency data not used; maximumFractionDigits should match minimumFractionDigits");
