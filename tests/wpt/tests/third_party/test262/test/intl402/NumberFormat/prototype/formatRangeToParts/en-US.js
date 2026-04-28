// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.NumberFormat-formatRangeToParts
description: Basic tests for the en-US output of formatRangeToParts()
locale: [en-US]
features: [Intl.NumberFormat-v3]
includes: [propertyHelper.js]
---*/

// Utils functions
function* zip(a, b) {
  assert.sameValue(a.length, b.length);
  for (let i = 0; i < a.length; ++i) {
    yield [i, a[i], b[i]];
  }
}

function compare(actual, expected) {
  for (const [i, actualEntry, expectedEntry] of zip(actual, expected)) {
    // assertions
    assert.sameValue(actualEntry.type, expectedEntry.type, `type for entry ${i}`);
    assert.sameValue(actualEntry.value, expectedEntry.value, `value for entry ${i}`);
    assert.sameValue(actualEntry.source, expectedEntry.source, `source for entry ${i}`);

    //  1.1.25_4.a  Let O be ObjectCreate(%ObjectPrototype%).
    assert.sameValue(Object.getPrototypeOf(actualEntry), Object.prototype, `prototype for entry ${i}`);
    //  1.1.25_4.b Perform ! CreateDataPropertyOrThrow(O, "type", part.[[Type]])
    verifyProperty(actualEntry, 'type', {  enumerable: true, writable: true, configurable: true });
    //  1.1.25_4.c Perform ! CreateDataPropertyOrThrow(O, "value", part.[[Value]]).
    verifyProperty(actualEntry, 'value', {  enumerable: true, writable: true, configurable: true });
    //  1.1.25_4.d Perform ! CreateDataPropertyOrThrow(O, "source", part.[[Source]]).
    verifyProperty(actualEntry, 'source', {  enumerable: true, writable: true, configurable: true });
  }
}

// Basic example test en-US
const nf = new Intl.NumberFormat("en-US", {
  style: "currency",
  currency: "USD",
  maximumFractionDigits: 0,
});

compare(nf.formatRangeToParts(3, 5), [
  {type: "currency", value: "$", source: "startRange"},
  {type: "integer", value: "3", source: "startRange"},
  {type: "literal", value: " – ", source: "shared"},
  {type: "currency", value: "$", source: "endRange"},
  {type: "integer", value: "5", source: "endRange"}
]);

compare(nf.formatRangeToParts(1, 1), [
  {type: 'approximatelySign', value: '~', source: 'shared'},
  {type: 'currency', value: '$', source: 'shared'},
  {type: 'integer', value: '1', source: 'shared'}
]);

compare(nf.formatRangeToParts(2.999, 3.001), [
  {type: 'approximatelySign', value: '~', source: 'shared'},
  {type: 'currency', value: '$', source: 'shared'},
  {type: 'integer', value: '3', source: 'shared'}
]);


