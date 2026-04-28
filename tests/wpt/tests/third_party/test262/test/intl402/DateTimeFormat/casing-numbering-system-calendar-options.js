// Copyright 2020 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
    Tests that the options numberingSystem and calendar are mapped
    to lower case properly.
author: Caio Lima
features: [Array.prototype.includes]
---*/

let defaultLocale = new Intl.DateTimeFormat().resolvedOptions().locale;

let supportedNumberingSystems = ["latn", "arab"].filter(nu =>
  new Intl.DateTimeFormat(defaultLocale + "-u-nu-" + nu)
    .resolvedOptions().numberingSystem === nu
);

if (supportedNumberingSystems.includes("latn")) {
  let dateTimeFormat = new Intl.DateTimeFormat(defaultLocale + "-u-nu-lATn");
  assert.sameValue(dateTimeFormat.resolvedOptions().numberingSystem, "latn", "Numbering system option should be in lower case");
}

if (supportedNumberingSystems.includes("arab")) {
  let dateTimeFormat = new Intl.DateTimeFormat(defaultLocale + "-u-nu-Arab");
  assert.sameValue(dateTimeFormat.resolvedOptions().numberingSystem, "arab", "Numbering system option should be in lower case");
}

let supportedCalendars = ["gregory", "chinese"].filter(ca =>
  new Intl.DateTimeFormat(defaultLocale + "-u-ca-" + ca)
    .resolvedOptions().calendar === ca
);

if (supportedCalendars.includes("gregory")) {
  let dateTimeFormat = new Intl.DateTimeFormat(defaultLocale + "-u-ca-Gregory");
  assert.sameValue(dateTimeFormat.resolvedOptions().calendar, "gregory", "Calendar option should be in lower case");
}

if (supportedCalendars.includes("chinese")) {
  let dateTimeFormat = new Intl.DateTimeFormat(defaultLocale + "-u-ca-CHINESE");
  assert.sameValue(dateTimeFormat.resolvedOptions().calendar, "chinese", "Calendar option should be in lower case");
}

