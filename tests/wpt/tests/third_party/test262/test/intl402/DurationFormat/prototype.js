// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Intl.DurationFormat instance object is created from %DurationFormatPrototype%.
info: |
    Intl.DurationFormat ([ locales [ , options ]])

    2. Let durationFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%DurationFormatPrototype%", « [[InitializedDurationFormat]], [[Locale]], [[DataLocale]], [[NumberingSystem]], [[Style]], [[YearsStyle]], [[YearsDisplay]], [[MonthsStyle]], [[MonthsDisplay]] , [[WeeksStyle]], [[WeeksDisplay]] , [[DaysStyle]], [[DaysDisplay]] , [[HoursStyle]], [[HoursDisplay]] , [[MinutesStyle]], [[MinutesDisplay]] , [[SecondsStyle]], [[SecondsDisplay]] , [[MillisecondsStyle]], [[MillisecondsDisplay]] , [[MicrosecondsStyle]], [[MicrosecondsDisplay]] , [[NanosecondsStyle]], [[NanosecondsDisplay]], [[FractionalDigits]] »).

features: [Intl.DurationFormat]
---*/

const value = new Intl.DurationFormat();
assert.sameValue(
  Object.getPrototypeOf(value),
  Intl.DurationFormat.prototype,
  "Object.getPrototypeOf(value) equals the value of Intl.DurationFormat.prototype"
);
