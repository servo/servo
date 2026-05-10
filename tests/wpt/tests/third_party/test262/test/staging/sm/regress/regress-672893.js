/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  pending
esid: pending
---*/
function f() {
    return function () { return function () { return function () {
    return function () { return function () { return function () {
    return function () { return function () { return function () {
    return function () { return function () { return function () {
    return function () { return function () { return function (a) {
        var v = a;
	assert.sameValue(v, 42);
	return function() { return v; };
    }; }; }; }; }; }; }; }; }; }; }; }; }; }; };
};

assert.sameValue(f()()()()()()()()()()()()()()()(42)(), 42);

