// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// MakeDay: Adding finite |year| and |month| can result in non-finite intermediate result.

assert.sameValue(Date.UTC(Number.MAX_VALUE, Number.MAX_VALUE), NaN);
assert.sameValue(new Date(Number.MAX_VALUE, Number.MAX_VALUE).getTime(), NaN);

// https://github.com/tc39/ecma262/issues/1087

var d = new Date(0);
d.setUTCFullYear(Number.MAX_VALUE, Number.MAX_VALUE);
assert.sameValue(d.getTime(), NaN);

