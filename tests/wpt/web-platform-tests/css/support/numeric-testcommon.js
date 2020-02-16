'use strict';

/*
Tests to verify that numeric values
(math functions, generally),
are handled correctly.

Relies on a #target element existing in the document,
as this might rely on layout to resolve styles,
and so it needs to be in the document.
*/


/*
By default, assumes testString evaluates to a <length>.
If this isn't true, override {base, prop} accordingly.
*/
function test_math_used(testString, expectedString, {base="123px", msg, prop="left"}={}) {
    const testEl = document.getElementById('target');
    if(testEl == null) throw "Couldn't find #target element to run tests on."
    test(()=>{
        testEl.style[prop] = base;
        testEl.style[prop] = testString;
        const usedValue = getComputedStyle(testEl)[prop];
        assert_not_equals(usedValue, base, `${testString} isn't valid in '${prop}'; got the default value instead.`);
        testEl.style[prop] = base;
        testEl.style[prop] = expectedString;
        const expectedValue = getComputedStyle(testEl)[prop];
        assert_not_equals(expectedValue, base, `${testString} isn't valid in '${prop}'; got the default value instead.`)
        assert_equals(usedValue, expectedValue, `${testString} and ${expectedString} serialize to the same thing in used values.`);
    }, msg || `${testString} should be used-value-equivalent to ${expectedString}`);
}

/*
All of these expect the testString to evaluate to a <number>.
*/
function test_plus_infinity(testString) {
    test_math_used(`calc(1px * ${testString})`, "calc(infinity * 1px)");
}
function test_minus_infinity(testString) {
    test_math_used(`calc(1px * ${testString})`, "calc(-infinity * 1px)");
}
function test_plus_zero(testString) {
    test_math_used(`calc(1px / ${testString})`, "calc(infinity * 1px)");
}
function test_minus_zero(testString) {
    test_math_used(`calc(1px / ${testString})`, "calc(-infinity * 1px)");
}
function test_nan(testString) {
    // Make sure that it's NaN, not an infinity,
    // by making sure that it's the same value both pos and neg.
    test_math_used(`calc(1px * ${testString})`, "calc(NaN * 1px)");
    test_math_used(`calc(-1px * ${testString})`, "calc(NaN * 1px)");
}