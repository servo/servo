/*---
defines: [ISOFields, assertSameISOFields]
---*/

function ISOFields(monthDay) {
  let re = /^(?<year>-?\d{4,6})-(?<month>\d{2})-(?<day>\d{2})\[u-ca=(?<calendar>[\w\-]+)\]$/;

  let str = monthDay.toString({calendarName: "always"});
  let match = str.match(re);
  assert.sameValue(match !== null, true, `can't match: ${str}`);

  let {year, month, day, calendar} = match.groups;
  let isoYear = Number(year);
  let isoMonth = Number(month);
  let isoDay = Number(day);

  let date = Temporal.PlainDate.from(str);
  let isoDate = date.withCalendar("iso8601");

  assert.sameValue(calendar, date.calendarId);
  assert.sameValue(isoYear, isoDate.year);
  assert.sameValue(isoMonth, isoDate.month);
  assert.sameValue(isoDay, isoDate.day);

  return {
    isoYear,
    isoMonth,
    isoDay,
    calendar,
  };
}

function assertSameISOFields(actual, expected) {
  let actualFields = ISOFields(actual);
  let expectedFields = ISOFields(expected);

  assert.sameValue(typeof actualFields.isoYear, "number");
  assert.sameValue(typeof actualFields.isoMonth, "number");
  assert.sameValue(typeof actualFields.isoDay, "number");

  assert.sameValue(actualFields.isoMonth > 0, true);
  assert.sameValue(actualFields.isoDay > 0, true);

  assert.sameValue(actualFields.isoYear, expectedFields.isoYear);
  assert.sameValue(actualFields.isoMonth, expectedFields.isoMonth);
  assert.sameValue(actualFields.isoDay, expectedFields.isoDay);
  assert.sameValue(actualFields.calendar, expectedFields.calendar);
}
