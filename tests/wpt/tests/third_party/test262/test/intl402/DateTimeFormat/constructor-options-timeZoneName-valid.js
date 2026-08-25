// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdatetimeformat
description: >
  Valid values for the `timeZoneName` option of the DateTimeFormat constructor
features: [Intl.DateTimeFormat-extend-timezonename]
---*/

var dtf;

dtf = new Intl.DateTimeFormat('en', { timeZoneName: 'short' });
assert.sameValue(dtf.resolvedOptions().timeZoneName, 'short');

dtf = new Intl.DateTimeFormat('en', { timeZoneName: 'long' });
assert.sameValue(dtf.resolvedOptions().timeZoneName, 'long');

dtf = new Intl.DateTimeFormat('en', { timeZoneName: 'shortOffset' });
assert.sameValue(dtf.resolvedOptions().timeZoneName, 'shortOffset');

dtf = new Intl.DateTimeFormat('en', { timeZoneName: 'longOffset' });
assert.sameValue(dtf.resolvedOptions().timeZoneName, 'longOffset');

dtf = new Intl.DateTimeFormat('en', { timeZoneName: 'shortGeneric' });
assert.sameValue(dtf.resolvedOptions().timeZoneName, 'shortGeneric');

dtf = new Intl.DateTimeFormat('en', { timeZoneName: 'longGeneric' });
assert.sameValue(dtf.resolvedOptions().timeZoneName, 'longGeneric');
