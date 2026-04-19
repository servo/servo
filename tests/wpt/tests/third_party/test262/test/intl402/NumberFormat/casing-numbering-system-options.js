// Copyright 2020 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
    Tests that the options numberingSystem are mapped to lower case.
author: Caio Lima
features: [Array.prototype.includes]
---*/

let defaultLocale = new Intl.NumberFormat().resolvedOptions().locale;

let supportedNumberingSystems = ["latn", "arab"].filter(nu =>
  new Intl.NumberFormat(defaultLocale + "-u-nu-" + nu)
    .resolvedOptions().numberingSystem === nu
);

if (supportedNumberingSystems.includes("latn")) {
  let numberFormat = new Intl.NumberFormat(defaultLocale + "-u-nu-lATn");
  assert.sameValue(numberFormat.resolvedOptions().numberingSystem, "latn", "Numbering system option should be in lower case");
}

if (supportedNumberingSystems.includes("arab")) {
  let numberFormat = new Intl.NumberFormat(defaultLocale + "-u-nu-Arab");
  assert.sameValue(numberFormat.resolvedOptions().numberingSystem, "arab", "Numbering system option should be in lower case");
}
