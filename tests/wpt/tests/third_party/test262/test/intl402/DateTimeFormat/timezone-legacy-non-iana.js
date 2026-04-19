// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializedatetimeformat
description: Only IANA time zone identifiers are allowed.
---*/

// List of non-IANA link names, copied from:
// https://github.com/unicode-org/icu/blob/main/icu4c/source/tools/tzcode/icuzones
const invalidTimeZones = [
  "ACT",
  "AET",
  "AGT",
  "ART",
  "AST",
  "BET",
  "BST",
  "CAT",
  "CNT",
  "CST",
  "CTT",
  "EAT",
  "ECT",
  "IET",
  "IST",
  "JST",
  "MIT",
  "NET",
  "NST",
  "PLT",
  "PNT",
  "PRT",
  "PST",
  "SST",
  "VST",
];

for (let timeZone of invalidTimeZones) {
  assert.throws(RangeError, () => {
    new Intl.DateTimeFormat(undefined, {timeZone});
  }, "Time zone: " + timeZone);
}
