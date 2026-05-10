// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Check all the algorithms that call ToPrimitive. Confirm that they're passing
// the correct hint, per spec.

var STRING = "xyzzy";
var NUMBER = 42;

function assertCallsToPrimitive(f, expectedHint, expectedResult) {
    var hint = undefined;
    var testObj = {
        [Symbol.toPrimitive](h) {
            assert.sameValue(hint, undefined);
            hint = h;
            return h === "number" ? NUMBER : STRING;
        }
    };
    var result = f(testObj);
    assert.sameValue(hint, expectedHint, String(f));
    assert.sameValue(result, expectedResult, String(f));
}

// ToNumber
assertCallsToPrimitive(Number, "number", NUMBER);

// ToString
assertCallsToPrimitive(String, "string", STRING);

// ToPropertyKey
var obj = {[STRING]: "pass"};
assertCallsToPrimitive(key => obj[key], "string", "pass");

// Abstract Relational Comparison
assertCallsToPrimitive(x => x >= 42, "number", true);
assertCallsToPrimitive(x => x > "42", "number", false);

// Abstract Equality Comparison
assertCallsToPrimitive(x => x != STRING, "default", false);
assertCallsToPrimitive(x => STRING == x, "default", true);
assertCallsToPrimitive(x => x == NUMBER, "default", false);
assertCallsToPrimitive(x => NUMBER != x, "default", true);

// Addition
assertCallsToPrimitive(x => 1 + x, "default", "1" + STRING);
assertCallsToPrimitive(x => "" + x, "default", STRING);

// Date constructor
assertCallsToPrimitive(x => (new Date(x)).valueOf(), "default", Number(STRING));

// Date.prototype.toJSON
var expected = "a suffusion of yellow";
function testJSON(x) {
    x.toJSON = Date.prototype.toJSON;
    x.toISOString = function () { return expected; };
    return JSON.stringify(x);
}
assertCallsToPrimitive(testJSON, "number", JSON.stringify(expected));

