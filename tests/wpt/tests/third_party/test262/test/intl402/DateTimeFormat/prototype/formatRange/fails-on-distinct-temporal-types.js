// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.formatRange
description: formatRange fails if given arguments of different Temporal types
features: [Temporal]
---*/

const us = new Intl.DateTimeFormat('en-US');

const instances = {
  date: new Date(1580527800000),
  instant: new Temporal.Instant(0n),
  plaindate: new Temporal.PlainDate(2000, 5, 2),
  plaindatetime: new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 654, 321),
  plainmonthday: new Temporal.PlainMonthDay(5, 2),
  plaintime: new Temporal.PlainTime(13, 37),
  plainyearmonth: new Temporal.PlainYearMonth(2019, 6),
  zoneddatetime: new Temporal.ZonedDateTime(0n, 'America/Kentucky/Louisville')
};

Object.entries(instances).forEach(([typeName, instance]) => {
  Object.entries(instances).forEach(([anotherTypeName, anotherInstance]) => {
    if (typeName !== anotherTypeName) {
      assert.throws(
        TypeError,
        () => { us.formatRange(instance, anotherInstance); },
        'formatRange: bad arguments (' + typeName + ' and ' + anotherTypeName + ')'
      );
    }
  });
});
