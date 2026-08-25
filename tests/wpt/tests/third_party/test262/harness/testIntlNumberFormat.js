// Copyright 2026 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: |
    Collection of functions used to assert the correctness of Intl.NumberFormat objects.
defines: [getUnitSeparators]
---*/

/**
 * @description Determine the unit/number separator(s) used by an
 *   Intl.NumberFormat implementation, which are not specified by ECMA-402 and
 *   generally come from CLDR (and observable in e.g. Japanese long display
 *   `時速 2 キロメートル` vs. `時速2キロメートル`).
 * @param {string} locale
 * @param {'narrow' | 'short' | 'long'} unitDisplay
 * @return {{ prefix: string, suffix: string }}
 */
function getUnitSeparators(locale, unitDisplay) {
  var nf = new Intl.NumberFormat(locale, { style: "unit", unit: "kilometer-per-hour", unitDisplay: unitDisplay });
  var parts = nf.formatToParts(1);
  var unitIndices = [];
  for (var i = 0; i < parts.length; i++) {
    if (parts[i].type === "unit") unitIndices.push(i);
  }

  var prefix = unitIndices[0] === 0 && parts[1] && parts[1].type === "literal"
    ? parts[1].value
    : "";

  var lastUnitIndex = unitIndices[unitIndices.length - 1];
  var suffix = lastUnitIndex === parts.length - 1 && parts[lastUnitIndex - 1] && parts[lastUnitIndex - 1].type === "literal"
    ? parts[lastUnitIndex - 1].value
    : "";

  return { prefix: prefix, suffix: suffix };
}
