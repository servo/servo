// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/

var x;

// check an empty object, just for sanity
var emptyObject = "{}";
x = JSON.parse(emptyObject);
assert.sameValue(typeof x, "object");
assert.sameValue(x instanceof Object, true);

x = JSON.parse(emptyObject);
assert.sameValue(typeof x, "object");

// booleans and null
x = JSON.parse("true");
assert.sameValue(x, true);

x = JSON.parse("true          ");
assert.sameValue(x, true);

x = JSON.parse("false");
assert.sameValue(x, false);

x = JSON.parse("           null           ");
assert.sameValue(x, null);

// numbers
x = JSON.parse("1234567890");
assert.sameValue(x, 1234567890);

x = JSON.parse("-9876.543210");
assert.sameValue(x, -9876.543210);

x = JSON.parse("0.123456789e-12");
assert.sameValue(x, 0.123456789e-12);

x = JSON.parse("1.234567890E+34");
assert.sameValue(x, 1.234567890E+34);

x = JSON.parse("      23456789012E66          \r\r\r\r      \n\n\n\n ");
assert.sameValue(x, 23456789012E66);

// strings
x = JSON.parse('"foo"');
assert.sameValue(x, "foo");

x = JSON.parse('"\\r\\n"');
assert.sameValue(x, "\r\n");

x = JSON.parse('  "\\uabcd\uef4A"');
assert.sameValue(x, "\uabcd\uef4A");

x = JSON.parse('"\\uabcd"  ');
assert.sameValue(x, "\uabcd");

x = JSON.parse('"\\f"');
assert.sameValue(x, "\f");
