// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.NumberFormat-formatRange
description: Basic tests for the en-US output of formatRange()
locale: [en-US]
features: [Intl.NumberFormat-v3]
---*/

// Basic example test en-US
const nf = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  maximumFractionDigits: 0,
});

assert.sameValue(nf.formatRange(3, 5), "$3 – $5");
assert.sameValue(nf.formatRange(2.9, 3.1), "~$3");


// Basic example test en-US using signDisplay to always
const nf2 = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  signDisplay: "always",
});

assert.sameValue(nf2.formatRange(2.9, 3.1), "+$2.90–3.10");

// Basic example test en-US string formatting
const nf3 = new Intl.NumberFormat("en-US");
const string1 = "987654321987654321";
const string2 = "987654321987654322";

assert.sameValue(nf3.formatRange(string1, string2), "987,654,321,987,654,321–987,654,321,987,654,322");

