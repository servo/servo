// Copyright 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: >
  A valid offset time zone identifier may not include U+2212 MINUS SIGN
---*/

// Note: the first character of each of these strings is U+2122 MINUS SIGN
const invalidIDs = [
  '−0900',
  '−10:00',
  '−05',
];
invalidIDs.forEach((id) => {
  assert.throws(RangeError, () => new Intl.DateTimeFormat("en", { timeZone: id }));
});
