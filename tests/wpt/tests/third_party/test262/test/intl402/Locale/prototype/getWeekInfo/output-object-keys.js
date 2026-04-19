// Copyright 2021 Igalia, S.L. All rights reserved.
// Copyright 2021 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getWeekInfo
description: >
    Checks that the return value of Intl.Locale.prototype.getWeekInfo is an Object
    with the correct keys and properties.
info: |
  get Intl.Locale.prototype.getWeekInfo
  ...
  6. Perform ! CreateDataPropertyOrThrow(info, "firstDay", wi.[[FirstDay]]).
  7. Perform ! CreateDataPropertyOrThrow(info, "weekend", we).
  ...
  CreateDataProperty ( O, P, V )
  ...
  3. Let newDesc be the PropertyDescriptor { [[Value]]: V, [[Writable]]: true,
  [[Enumerable]]: true, [[Configurable]]: true }.
features: [Reflect,Intl.Locale,Intl.Locale-info]
includes: [propertyHelper.js, compareArray.js]
---*/

const result = new Intl.Locale('en').getWeekInfo();
function isIntegerBetweenOneAndSeven(value) {
  return value === 1 || value === 2 || value === 3 || value === 4 || value === 5 || value === 6 || value === 7;
}

assert.compareArray(Reflect.ownKeys(result), ['firstDay', 'weekend']);

verifyProperty(result, 'firstDay', {
  writable: true,
  enumerable: true,
  configurable: true
});
assert(
  isIntegerBetweenOneAndSeven(new Intl.Locale('en').getWeekInfo().firstDay),
  '`firstDay` must be an integer between one and seven (inclusive)'
);

verifyProperty(result, 'weekend', {
  writable: true,
  enumerable: true,
  configurable: true
});
assert(
  new Intl.Locale('en').getWeekInfo().weekend.every(isIntegerBetweenOneAndSeven),
  '`weekend` must include integers between one and seven (inclusive)'
);

let original = new Intl.Locale('en').getWeekInfo().weekend;
let sorted = original.slice().sort();
assert.compareArray(original, sorted);
