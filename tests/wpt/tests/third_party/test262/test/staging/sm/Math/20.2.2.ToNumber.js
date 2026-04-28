/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
/*
 *   ECMA-262 6th Edition / Draft November 8, 2013
 *
 *   20.2.2 Function Properties of the Math Object
 */

/*
 * This custom object will allow us to check if valueOf() is called
 */

TestNumber.prototype = new Number();

function TestNumber(value) {
    this.value = value;
    this.valueOfCalled = false;
}

TestNumber.prototype = {
    valueOf: function() {
        this.valueOfCalled = true;
        return this.value;
    }
}

// Verify that each TestNumber's flag is set after calling Math func
function test(func /*, args */) {
    var args = Array.prototype.slice.call(arguments, 1);
    func.apply(null, args);

    for (var i = 0; i < args.length; ++i)
        assert.sameValue(args[i].valueOfCalled, true);
}

// Note that we are not testing these functions' return values
// We only test whether valueOf() is called for each argument

// 1. Test Math.atan2()
var x = new TestNumber(1);
test(Math.atan2, x);

var x = new TestNumber(1);
var y = new TestNumber(2);
test(Math.atan2, y, x);

// Remove comment block once patch for bug 896264 is approved
/*
// 2. Test Math.hypot()
var x = new TestNumber(1);
test(Math.hypot, x);

var x = new TestNumber(1);
var y = new TestNumber(2);
test(Math.hypot, x, y);

var x = new TestNumber(1);
var y = new TestNumber(2);
var z = new TestNumber(3);
test(Math.hypot, x, y, z);
*/

// Remove comment block once patch for bug 808148 is approved
/*
// 3. Test Math.imul()
var x = new TestNumber(1);
test(Math.imul, x);

var x = new TestNumber(1);
var y = new TestNumber(2);
test(Math.imul, x, y);
*/

// 4. Test Math.max()
var x = new TestNumber(1);
test(Math.max, x);

var x = new TestNumber(1);
var y = new TestNumber(2);
test(Math.max, x, y);

var x = new TestNumber(1);
var y = new TestNumber(2);
var z = new TestNumber(3);
test(Math.max, x, y, z);

// 5. Test Math.min()
var x = new TestNumber(1);
test(Math.min, x);

var x = new TestNumber(1);
var y = new TestNumber(2);
test(Math.min, x, y);

var x = new TestNumber(1);
var y = new TestNumber(2);
var z = new TestNumber(3);
test(Math.min, x, y, z);

// 6. Test Math.pow()
var x = new TestNumber(1);
test(Math.pow, x);

var x = new TestNumber(1);
var y = new TestNumber(2);
test(Math.pow, x, y);

