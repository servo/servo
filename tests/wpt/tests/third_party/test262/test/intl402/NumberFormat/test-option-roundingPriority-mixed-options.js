// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Tests that the digits are determined correctly when specifying at same time «"minimumFractionDigits", "maximumFractionDigits", "minimumSignificantDigits", "maximumSignificantDigits"»
features: [Intl.NumberFormat-v3]
includes: [testIntl.js]
---*/

const locales = [new Intl.NumberFormat().resolvedOptions().locale, 'ar', 'de', 'th', 'ja'];

const numberingSystems = ['arab', 'latn', 'thai', 'hanidec'];

const expectedResults = {
  'morePrecision': {
    'same-minimums': { '1': '1.0', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.0' },
    'same-maximums': { '1': '1', '1.500': '1.5', '1.625': '1.63', '1.750': '1.75', '1.875': '1.88', '2.000': '2' },
    'minSd-larger-minFd': { '1': '1.00', '1.500': '1.50', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.00' },
    'minSd-smaller-minFd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
    'minSd-smaller-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
    'minSd-larger-maxFd': { '1': '1.00', '1.500': '1.50', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.00' },
    'maxSd-larger-minFd': { '1': '1.0', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.0' },
    'maxSd-smaller-minFd': { '1': '1.000', '1.500': '1.500', '1.625': '1.625', '1.750': '1.750', '1.875': '1.875', '2.000': '2.000' },
    'maxSd-smaller-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
    'maxSd-larger-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.63', '1.750': '1.75', '1.875': '1.88', '2.000': '2' },
    'minSd-maxSd-smaller-minFd-maxFd': { '1': '1.000', '1.500': '1.500', '1.625': '1.625', '1.750': '1.750', '1.875': '1.875', '2.000': '2.000' },
    'minSd-maxSd-larger-minFd-maxFd': { '1': '1.00', '1.500': '1.50', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.00' },
    'same-minimums-maxFd-larger-maxSd': { '1': '1.0', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.0' },
    'same-minimums-maxFd-smaller-maxSd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
  },
  'lessPrecision': {
    'same-minimums': { '1': '1.00', '1.500': '1.50', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.00' },
    'same-maximums': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'minSd-larger-minFd': { '1': '1.0', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.0' },
    'minSd-smaller-minFd': { '1': '1.000', '1.500': '1.500', '1.625': '1.625', '1.750': '1.750', '1.875': '1.875', '2.000': '2.000' },
    'minSd-smaller-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
    'minSd-larger-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'maxSd-larger-minFd': { '1': '1', '1.500': '1.5', '1.625': '1.63', '1.750': '1.75', '1.875': '1.88', '2.000': '2' },
    'maxSd-smaller-minFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'maxSd-smaller-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'maxSd-larger-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'minSd-maxSd-smaller-minFd-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'minSd-maxSd-larger-minFd-maxFd': { '1': '1.0', '1.500': '1.5', '1.625': '1.63', '1.750': '1.75', '1.875': '1.88', '2.000': '2.0' },
    'same-minimums-maxFd-larger-maxSd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'same-minimums-maxFd-smaller-maxSd': { '1': '1.0', '1.500': '1.5', '1.625': '1.63', '1.750': '1.75', '1.875': '1.88', '2.000': '2.0' },
  },
  'auto': {
    'same-minimums': { '1': '1.0', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.0' },
    'same-maximums': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'minSd-larger-minFd': { '1': '1.00', '1.500': '1.50', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.00' },
    'minSd-smaller-minFd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
    'minSd-smaller-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
    'minSd-larger-maxFd': { '1': '1.00', '1.500': '1.50', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.00' },
    'maxSd-larger-minFd': { '1': '1', '1.500': '1.5', '1.625': '1.63', '1.750': '1.75', '1.875': '1.88', '2.000': '2' },
    'maxSd-smaller-minFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'maxSd-smaller-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'maxSd-larger-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.63', '1.750': '1.75', '1.875': '1.88', '2.000': '2' },
    'minSd-maxSd-smaller-minFd-maxFd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'minSd-maxSd-larger-minFd-maxFd': { '1': '1.00', '1.500': '1.50', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2.00' },
    'same-minimums-maxFd-larger-maxSd': { '1': '1', '1.500': '1.5', '1.625': '1.6', '1.750': '1.8', '1.875': '1.9', '2.000': '2' },
    'same-minimums-maxFd-smaller-maxSd': { '1': '1', '1.500': '1.5', '1.625': '1.625', '1.750': '1.75', '1.875': '1.875', '2.000': '2' },
  },
};

const optionsMatrix = {
  'same-minimums': { minimumSignificantDigits: 2, minimumFractionDigits: 2 },
  'same-maximums': { maximumSignificantDigits: 2, maximumFractionDigits: 2 },
  'minSd-larger-minFd': { minimumSignificantDigits: 3, minimumFractionDigits: 1 },
  'minSd-smaller-minFd': { minimumSignificantDigits: 1, minimumFractionDigits: 3 },
  'minSd-smaller-maxFd': { minimumSignificantDigits: 1, maximumFractionDigits: 3 },
  'minSd-larger-maxFd': { minimumSignificantDigits: 3, maximumFractionDigits: 1 },
  'maxSd-larger-minFd': { maximumSignificantDigits: 3, minimumFractionDigits: 1 },
  'maxSd-smaller-minFd': { maximumSignificantDigits: 2, minimumFractionDigits: 3 },
  'maxSd-smaller-maxFd': { maximumSignificantDigits: 2, maximumFractionDigits: 3 },
  'maxSd-larger-maxFd': { maximumSignificantDigits: 3, maximumFractionDigits: 1 },
  'minSd-maxSd-smaller-minFd-maxFd': { minimumSignificantDigits: 1, maximumSignificantDigits: 2, minimumFractionDigits: 3, maximumFractionDigits: 4 },
  'minSd-maxSd-larger-minFd-maxFd': { minimumSignificantDigits: 3, maximumSignificantDigits: 4, minimumFractionDigits: 1, maximumFractionDigits: 2 },
  'same-minimums-maxFd-larger-maxSd': { minimumSignificantDigits: 1, maximumSignificantDigits: 2, minimumFractionDigits: 1, maximumFractionDigits: 4 },
  'same-minimums-maxFd-smaller-maxSd': { minimumSignificantDigits: 1, maximumSignificantDigits: 4, minimumFractionDigits: 1, maximumFractionDigits: 2 },
};

function testPrecision(mode) {
  Object.keys(optionsMatrix).forEach((key) => {
  testNumberFormat(
      locales,
      numberingSystems,
      { ...optionsMatrix[key], roundingPriority: mode, userGrouping: false },
      expectedResults[mode][key]
  );
  });
}

['morePrecision', 'lessPrecision', 'auto'].forEach(testPrecision);
