// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-Intl.DisplayNames.prototype.of
description: Returns string value for valid `dateTimeField` codes
features: [Intl.DisplayNames-v2]
---*/

var displayNames = new Intl.DisplayNames(undefined, {type: 'dateTimeField'});

assert.sameValue(typeof displayNames.of('era'), 'string', 'era');
assert.sameValue(typeof displayNames.of('year'), 'string', 'year');
assert.sameValue(typeof displayNames.of('quarter'), 'string', 'quarter');
assert.sameValue(typeof displayNames.of('month'), 'string', 'month');
assert.sameValue(typeof displayNames.of('weekOfYear'), 'string', 'weekOfYear');
assert.sameValue(typeof displayNames.of('weekday'), 'string', 'weekday');
assert.sameValue(typeof displayNames.of('day'), 'string', 'day');
assert.sameValue(typeof displayNames.of('dayPeriod'), 'string', 'dayPeriod');
assert.sameValue(typeof displayNames.of('hour'), 'string', 'hour');
assert.sameValue(typeof displayNames.of('minute'), 'string', 'minute');
assert.sameValue(typeof displayNames.of('second'), 'string', 'second');
assert.sameValue(typeof displayNames.of('timeZoneName'), 'string', 'timeZoneName');
