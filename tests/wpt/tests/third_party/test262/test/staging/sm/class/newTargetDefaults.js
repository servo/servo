// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
// Check that new.target works properly in defaults.

function check(expected, actual = new.target) { assert.sameValue(actual, expected); }
new check(check);
check(undefined);

let evaldCheck = eval("(" + check.toString() + ")");
new evaldCheck(evaldCheck);
evaldCheck(undefined);

function testInFunction() {
    function checkInFunction(expected, actual = new.target) { assert.sameValue(actual, expected); }
    new checkInFunction(checkInFunction);
    checkInFunction(undefined);

    let evaldCheckInFunction = eval("(" + checkInFunction.toString() + ")");
    new evaldCheckInFunction(evaldCheckInFunction);
    evaldCheckInFunction(undefined);
}

testInFunction();
new testInFunction();

