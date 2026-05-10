// Copyright 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.resolvedOptions
description: >
  Intl.DateTimeFormat.prototype.resolvedOptions properly
  reflect hourCycle settings.
info: |
  12.4.5 Intl.DateTimeFormat.prototype.resolvedOptions()

includes: [propertyHelper.js]
features: [Array.prototype.includes]
---*/

/* Values passed via unicode extension key work */

const hcValues = ['h11', 'h12', 'h23', 'h24'];
const hour12Values = ['h11', 'h12'];

const dataPropertyDesc = { writable: true, enumerable: true, configurable: true };

for (const hcValue of hcValues) {
  const resolvedOptions = new Intl.DateTimeFormat(`de-u-hc-${hcValue}`, {
    hour: 'numeric'
  }).resolvedOptions();

  assert.sameValue(resolvedOptions.hourCycle, hcValue);
  assert.sameValue(resolvedOptions.hour12, hour12Values.includes(hcValue));

  verifyProperty(resolvedOptions, 'hourCycle', dataPropertyDesc);
  verifyProperty(resolvedOptions, 'hour12', dataPropertyDesc);
}

/* Values passed via options work */

for (const hcValue of hcValues) {
  const resolvedOptions = new Intl.DateTimeFormat(`en-US`, {
    hour: 'numeric',
    hourCycle: hcValue
  }).resolvedOptions();

  assert.sameValue(resolvedOptions.hourCycle, hcValue);
  assert.sameValue(resolvedOptions.hour12, hour12Values.includes(hcValue));

  verifyProperty(resolvedOptions, 'hourCycle', dataPropertyDesc);
  verifyProperty(resolvedOptions, 'hour12', dataPropertyDesc);
}

/* When both extension key and option is passed, option takes precedence */

let resolvedOptions = new Intl.DateTimeFormat(`en-US-u-hc-h12`, {
  hour: 'numeric',
  hourCycle: 'h23'
}).resolvedOptions();

assert.sameValue(resolvedOptions.hourCycle, 'h23');
assert.sameValue(resolvedOptions.hour12, false);

verifyProperty(resolvedOptions, 'hourCycle', dataPropertyDesc);
verifyProperty(resolvedOptions, 'hour12', dataPropertyDesc);

/* When hour12 and hourCycle are set, hour12 takes precedence */

resolvedOptions = new Intl.DateTimeFormat(`fr`, {
  hour: 'numeric',
  hour12: true,
  hourCycle: 'h23'
}).resolvedOptions();

assert(hour12Values.includes(resolvedOptions.hourCycle));
assert.sameValue(resolvedOptions.hour12, true);

verifyProperty(resolvedOptions, 'hourCycle', dataPropertyDesc);
verifyProperty(resolvedOptions, 'hour12', dataPropertyDesc);

/* When hour12 and extension key are set, hour12 takes precedence */

resolvedOptions = new Intl.DateTimeFormat(`fr-u-hc-h24`, {
  hour: 'numeric',
  hour12: true,
}).resolvedOptions();

assert(hour12Values.includes(resolvedOptions.hourCycle));
assert.sameValue(resolvedOptions.hour12, true);

verifyProperty(resolvedOptions, 'hourCycle', dataPropertyDesc);
verifyProperty(resolvedOptions, 'hour12', dataPropertyDesc);

/* When the hour is not in the pattern, hourCycle and hour12 are not defined. */

resolvedOptions = new Intl.DateTimeFormat("fr", {
  hourCycle: "h12",
  hour12: false,
}).resolvedOptions();

assert.sameValue(resolvedOptions.hour, undefined,
                 "Precondition: hour should not be included by default");
assert.sameValue(resolvedOptions.hourCycle, undefined);
assert.sameValue(resolvedOptions.hour12, undefined);
