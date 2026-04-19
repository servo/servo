// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Tests for existance and behavior of Intl.DateTimeFormat.prototype.formatToParts
features: [Array.prototype.includes]
---*/

function reduce(parts) {
  return parts.map(part => part.value).join('');
}

function compareFTPtoFormat(locales, options, value) {
  const dtf = new Intl.DateTimeFormat(locales, options);
  assert.sameValue(
    dtf.format(value),
    reduce(dtf.formatToParts(value)),
    `Expected the same value for value ${value},
     locales: ${locales} and options: ${options}`
  );
}

compareFTPtoFormat();
compareFTPtoFormat('pl');
compareFTPtoFormat(['pl']);
compareFTPtoFormat([]);
compareFTPtoFormat(['de'], undefined, 0);
compareFTPtoFormat(['de'], undefined, -10);
compareFTPtoFormat(['de'], undefined, 25324234235);
compareFTPtoFormat(['de'], {
  day: '2-digit'
}, Date.now());
compareFTPtoFormat(['de'], {
  day: 'numeric',
  year: '2-digit'
}, Date.now());
compareFTPtoFormat(['ar'], {
  month: 'numeric',
  day: 'numeric',
  year: '2-digit'
}, Date.now());

const actualPartTypes = new Intl.DateTimeFormat('en-us', {
  weekday: 'long',
  era: 'long',
  year: 'numeric',
  month: 'numeric',
  day: 'numeric',
  hour: 'numeric',
  minute: 'numeric',
  second: 'numeric',
  hour12: true,
  timeZone: 'UTC',
  timeZoneName: 'long'
}).formatToParts(Date.UTC(2012, 11, 17, 3, 0, 42))
  .map(part => part.type);

const legalPartTypes = [
  'weekday',
  'era',
  'year',
  'month',
  'day',
  'hour',
  'minute',
  'second',
  'literal',
  'dayPeriod',
  'timeZoneName',
];

actualPartTypes.forEach(function(type) {
  assert(legalPartTypes.includes(type), `${type} is not a legal type`);
});
