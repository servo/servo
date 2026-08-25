// Copyright 2019 Mozilla Corporation, Igalia S.L. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.resolvedOptions
description: >
  Intl.DateTimeFormat.prototype.resolvedOptions properly
  reflect hourCycle settings when using dateStyle.
  Note: "properly reflect hourCycle settings when using dateStyle", in this context, means "if dateStyle but not timeStyle is set, both hourCycle and hour12 will be *undefined*". This is because the CreateDateTimeFormat AO resets [[HourCycle]] to *undefined* if [[Hour]] is *undefined*, and if dateStyle but not timeStyle is set, [[HourCycle]] is set to *undefined*. 
info: |
  11.3.7 Intl.DateTimeFormat.prototype.resolvedOptions()
  ...
  5. For each row of Table 6, except the header row, in table order, do
    a. Let p be the Property value of the current row.
    b. If p is "hour12", then
        i. Let hc be dtf.[[HourCycle]].
        ii. If hc is "h11" or "h12", let v be true.
        iii. Else if, hc is "h23" or "h24", let v be false.
        iv. Else, let v be undefined.
    c. Else,
        i. Let v be the value of dtf's internal slot whose name is the Internal Slot value of the current row.
    d. If the Internal Slot value of the current row is an Internal Slot value in Table 7, then
        i. If dtf.[[DateStyle]] is not undefined or dtf.[[TimeStyle]] is not undefined, then
            1. Let v be undefined.
    e. If v is not undefined, then
        i. Perform ! CreateDataPropertyOrThrow(options, p, v).

  11.1.2 CreateDateTimeFormat( newTarget, locales, options, required, defaults)
  ...
  45. If dateTimeFormat.[[Hour]] is undefined, then

    a. Set dateTimeFormat.[[HourCycle]] to undefined.
features: [Intl.DateTimeFormat-datetimestyle]
---*/

const hcValues = ["h11", "h12", "h23", "h24"];
const hour12Values = ["h11", "h12"];

for (const dateStyle of ["full", "long", "medium", "short"]) {
  assert.sameValue(new Intl.DateTimeFormat([], { dateStyle }).resolvedOptions().dateStyle,
                   dateStyle,
                   `Should support dateStyle=${dateStyle}`);

  /* Values passed via unicode extension key set to *undefined* */

  for (const hcValue of hcValues) {
    const resolvedOptions = new Intl.DateTimeFormat(`de-u-hc-${hcValue}`, {
      dateStyle,
    }).resolvedOptions();

    assert.sameValue(resolvedOptions.hourCycle, undefined);
    assert.sameValue(resolvedOptions.hour12, undefined);
  }

  /* Values passed via options set to *undefined**/

  for (const hcValue of hcValues) {
    const resolvedOptions = new Intl.DateTimeFormat("en-US", {
      dateStyle,
      hourCycle: hcValue
    }).resolvedOptions();

    assert.sameValue(resolvedOptions.hourCycle, undefined);
    assert.sameValue(resolvedOptions.hour12, undefined);
  }

  let resolvedOptions = new Intl.DateTimeFormat("en-US-u-hc-h12", {
    dateStyle,
    hourCycle: "h23"
  }).resolvedOptions();

  assert.sameValue(resolvedOptions.hourCycle, undefined);
  assert.sameValue(resolvedOptions.hour12, undefined);

  resolvedOptions = new Intl.DateTimeFormat("fr", {
    dateStyle,
    hour12: true,
    hourCycle: "h23"
  }).resolvedOptions();

  assert.sameValue(resolvedOptions.hourCycle, undefined);
  assert.sameValue(resolvedOptions.hour12, undefined);

  resolvedOptions = new Intl.DateTimeFormat("fr-u-hc-h24", {
    dateStyle,
    hour12: true,
  }).resolvedOptions();

  assert.sameValue(resolvedOptions.hourCycle, undefined);
  assert.sameValue(resolvedOptions.hour12, undefined);
}
