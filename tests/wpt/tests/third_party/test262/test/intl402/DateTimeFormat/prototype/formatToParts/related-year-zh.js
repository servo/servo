// Copyright 2019 Google Inc, Igalia S.L. All rights reserved.
// Copyright 2020 Apple Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  Checks the output of 'relatedYear' and 'yearName' type, and
  the choice of pattern based on calendar.
locale: [zh-u-ca-chinese]
---*/

const df = new Intl.DateTimeFormat("zh-u-ca-chinese", {year: "numeric"});
const date = new Date(2019, 5, 1);
const actual = df.formatToParts(date);

const expected = [
  {type: "relatedYear", value: "2019"},
  {type: "yearName", value: "己亥"},
  {type: "literal", value: "年"},
];

assert.sameValue(Array.isArray(actual), true, 'actual is Array');

if (actual.length <= 2) {
  expected.shift(); // removes the relatedYear
}

actual.forEach(({ type, value }, i) => {
  const { type: eType, value: eValue } = expected[i];
  assert.sameValue(type, eType, `actual[${i}].type should be ${eType}`);
  assert.sameValue(value, eValue, `actual[${i}].value should be ${eValue}`);
});
