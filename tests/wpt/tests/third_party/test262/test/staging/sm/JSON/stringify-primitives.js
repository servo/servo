// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

// sanity
var x = JSON.stringify({});
assert.sameValue(x, "{}");

// booleans and null
x = JSON.stringify(true);
assert.sameValue(x, "true");

x = JSON.stringify(false);
assert.sameValue(x, "false");

x = JSON.stringify(new Boolean(false));
assert.sameValue(x, "false");

x = JSON.stringify(null);
assert.sameValue(x, "null");

x = JSON.stringify(1234);
assert.sameValue(x, "1234");

x = JSON.stringify(new Number(1234));
assert.sameValue(x, "1234");

x = JSON.stringify("asdf");
assert.sameValue(x, '"asdf"');

x = JSON.stringify(new String("asdf"));
assert.sameValue(x, '"asdf"');

assert.sameValue(JSON.stringify(undefined), undefined);
assert.sameValue(JSON.stringify(function(){}), undefined);
assert.sameValue(JSON.stringify(JSON.stringify), undefined);
