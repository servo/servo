// Copyright 2013 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.1.1_32
description: >
    Tests that the options minimumSignificantDigits and
    maximumSignificantDigits are read in the right sequence.
author: Norbert Lindenberg
---*/

var minimumSignificantDigitsRead = false;
var maximumSignificantDigitsRead = false;

function readMinimumSignificantDigits() {
    assert.sameValue(minimumSignificantDigitsRead, false,
                     "minimumSignificantDigits getter already called");
    assert.sameValue(maximumSignificantDigitsRead, false,
                     "maximumSignificantDigits getter called before minimumSignificantDigits");
    minimumSignificantDigitsRead = true;
    return 1;
}

function readMaximumSignificantDigits() {
    assert.sameValue(maximumSignificantDigitsRead, false,
                     "maximumSignificantDigits getter already called");
    maximumSignificantDigitsRead = true;
    return 1;
}

var options = {};
Object.defineProperty(options, "minimumSignificantDigits",
    { get: readMinimumSignificantDigits });
Object.defineProperty(options, "maximumSignificantDigits",
    { get: readMaximumSignificantDigits });

new Intl.NumberFormat("de", options);

assert(minimumSignificantDigitsRead, "minimumSignificantDigits getter was called once");
assert(maximumSignificantDigitsRead, "maximumSignificantDigits getter was called once");
