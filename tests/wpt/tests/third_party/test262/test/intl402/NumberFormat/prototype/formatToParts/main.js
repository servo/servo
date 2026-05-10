// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Tests for existance and behavior of Intl.NumberFormat.prototype.formatToParts
---*/

function reduce(parts) {
  return parts.map(part => part.value).join('');
}

function compareFTPtoFormat(locales, options, value) {
  const nf = new Intl.NumberFormat(locales, options);
  assert.sameValue(
    nf.format(value),
    reduce(nf.formatToParts(value)),
    `Expected the same value for value ${value},
     locales: ${locales} and options: ${options}`
  );
}

const num1 = 123456.789;
const num2 = 0.123;

compareFTPtoFormat();
compareFTPtoFormat('pl');
compareFTPtoFormat(['pl']);
compareFTPtoFormat([]);
compareFTPtoFormat(['de'], undefined, 0);
compareFTPtoFormat(['de'], undefined, -10);
compareFTPtoFormat(['de'], undefined, 25324234235);
compareFTPtoFormat(['de'], undefined, num1);
compareFTPtoFormat(['de'], {
  style: 'percent'
}, num2);
compareFTPtoFormat(['de'], {
  style: 'currency',
  currency: 'EUR'
}, num1);
compareFTPtoFormat(['de'], {
  style: 'currency',
  currency: 'EUR',
  currencyDisplay: 'code'
}, num1);
compareFTPtoFormat(['de'], {
  useGrouping: true
}, num1);
compareFTPtoFormat(['de'], {
  useGrouping: false
}, num1);
compareFTPtoFormat(['de'], {
  minimumIntegerDigits: 2
}, num2);
compareFTPtoFormat(['de'], {
  minimumFractionDigits: 6
}, num2);
compareFTPtoFormat(['de'], {
  maximumFractionDigits: 1
}, num2);
compareFTPtoFormat(['de'], {
  maximumSignificantDigits: 3
}, num1);
compareFTPtoFormat(['de'], {
  maximumSignificantDigits: 5
}, num1);
