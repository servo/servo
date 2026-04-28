// Copyright 2019 Mozilla Corporation, Igalia S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.resolvedOptions
description: >
  Intl.DateTimeFormat.prototype.resolvedOptions properly
  reflect hourCycle settings when using timeStyle.
includes: [propertyHelper.js]
features: [Intl.DateTimeFormat-datetimestyle, Array.prototype.includes]
---*/

const hcValues = ["h11", "h12", "h23", "h24"];
const hour12Values = ["h11", "h12"];
const dataPropertyDesc = { writable: true, enumerable: true, configurable: true };

for (const timeStyle of ["full", "long", "medium", "short"]) {
  assert.sameValue(new Intl.DateTimeFormat([], { timeStyle }).resolvedOptions().timeStyle,
                   timeStyle,
                   `Should support timeStyle=${timeStyle}`);

  /* Values passed via unicode extension key work */

  for (const hcValue of hcValues) {
    const resolvedOptions = new Intl.DateTimeFormat(`de-u-hc-${hcValue}`, {
      timeStyle,
    }).resolvedOptions();

    assert.sameValue(resolvedOptions.hourCycle, hcValue);
    assert.sameValue(resolvedOptions.hour12, hour12Values.includes(hcValue));
  }

  /* Values passed via options work */

  for (const hcValue of hcValues) {
    const resolvedOptions = new Intl.DateTimeFormat("en-US", {
      timeStyle,
      hourCycle: hcValue
    }).resolvedOptions();

    assert.sameValue(resolvedOptions.hourCycle, hcValue);
    assert.sameValue(resolvedOptions.hour12, hour12Values.includes(hcValue));

    verifyProperty(resolvedOptions, "hourCycle", dataPropertyDesc);
    verifyProperty(resolvedOptions, "hour12", dataPropertyDesc);
  }

  /* When both extension key and option is passed, option takes precedence */

  let resolvedOptions = new Intl.DateTimeFormat("en-US-u-hc-h12", {
    timeStyle,
    hourCycle: "h23"
  }).resolvedOptions();

  assert.sameValue(resolvedOptions.hourCycle, "h23");
  assert.sameValue(resolvedOptions.hour12, false);

  verifyProperty(resolvedOptions, "hourCycle", dataPropertyDesc);
  verifyProperty(resolvedOptions, "hour12", dataPropertyDesc);

  /* When hour12 and hourCycle are set, hour12 takes precedence */

  resolvedOptions = new Intl.DateTimeFormat("fr", {
    timeStyle,
    hour12: true,
    hourCycle: "h23"
  }).resolvedOptions();

  assert(hour12Values.includes(resolvedOptions.hourCycle));
  assert.sameValue(resolvedOptions.hour12, true);

  verifyProperty(resolvedOptions, "hourCycle", dataPropertyDesc);
  verifyProperty(resolvedOptions, "hour12", dataPropertyDesc);

  /* When hour12 and extension key are set, hour12 takes precedence */

  resolvedOptions = new Intl.DateTimeFormat("fr-u-hc-h24", {
    timeStyle,
    hour12: true,
  }).resolvedOptions();

  assert(hour12Values.includes(resolvedOptions.hourCycle));
  assert.sameValue(resolvedOptions.hour12, true);

  verifyProperty(resolvedOptions, "hourCycle", dataPropertyDesc);
  verifyProperty(resolvedOptions, "hour12", dataPropertyDesc);
}
