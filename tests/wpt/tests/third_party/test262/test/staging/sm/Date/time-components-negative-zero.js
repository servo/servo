// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Don't return negative zero for get[Hours,Minutes,Seconds,Milliseconds] for dates before 1970.

let date = new Date(1955, 0, 1);
assert.sameValue(date.getTime() < 0, true);
assert.sameValue(date.getHours(), +0);
assert.sameValue(date.getMinutes(), +0);
assert.sameValue(date.getSeconds(), +0);
assert.sameValue(date.getMilliseconds(), +0);

let utc = new Date(Date.UTC(1955, 0, 1));
assert.sameValue(utc.getTime() < 0, true);
assert.sameValue(utc.getUTCHours(), +0);
assert.sameValue(utc.getUTCMinutes(), +0);
assert.sameValue(utc.getUTCSeconds(), +0);
assert.sameValue(utc.getUTCMilliseconds(), +0);

