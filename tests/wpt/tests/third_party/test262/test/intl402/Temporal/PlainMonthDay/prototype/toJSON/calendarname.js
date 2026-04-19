// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.protoype.tojson
description: toJSON doesn't take calendarName into account.
features: [Temporal]
---*/

const tests = [
  [[], "05-02"],
  [["gregory"], "1972-05-02[u-ca=gregory]"],
];
const options = {
  get calendarName() {
    TemporalHelpers.assertUnreachable("calendarName should not be accessed");
    return "";
  }
};

for (const [args, expected] of tests) {
  const monthday = new Temporal.PlainMonthDay(5, 2, ...args);
  const result = monthday.toJSON(options);
  assert.sameValue(result, expected);
}
