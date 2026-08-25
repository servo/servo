/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Stringification of Boolean/String/Number objects
info: bugzilla.mozilla.org/show_bug.cgi?id=584909
esid: pending
---*/

function redefine(obj, prop, fun)
{
  var desc =
    { value: fun, writable: true, configurable: true, enumerable: false };
  Object.defineProperty(obj, prop, desc);
}

assert.sameValue(JSON.stringify(new Boolean(false)), "false");

assert.sameValue(JSON.stringify(new Number(5)), "5");

assert.sameValue(JSON.stringify(new String("foopy")), '"foopy"');


var numToString = Number.prototype.toString;
var numValueOf = Number.prototype.valueOf;
var objToString = Object.prototype.toString;
var objValueOf = Object.prototype.valueOf;
var boolToString = Boolean.prototype.toString;
var boolValueOf = Boolean.prototype.valueOf;

redefine(Boolean.prototype, "toString", function() { return 17; });
assert.sameValue(JSON.stringify(new Boolean(false)), "false")
delete Boolean.prototype.toString;
assert.sameValue(JSON.stringify(new Boolean(false)), "false");
delete Object.prototype.toString;
assert.sameValue(JSON.stringify(new Boolean(false)), "false");
delete Boolean.prototype.valueOf;
assert.sameValue(JSON.stringify(new Boolean(false)), "false");
delete Object.prototype.valueOf;
assert.sameValue(JSON.stringify(new Boolean(false)), "false");


redefine(Boolean.prototype, "toString", boolToString);
redefine(Boolean.prototype, "valueOf", boolValueOf);
redefine(Object.prototype, "toString", objToString);
redefine(Object.prototype, "valueOf", objValueOf);

redefine(Number.prototype, "toString", function() { return 42; });
assert.sameValue(JSON.stringify(new Number(5)), "5");

redefine(Number.prototype, "valueOf", function() { return 17; });
assert.sameValue(JSON.stringify(new Number(5)), "17");

delete Number.prototype.toString;
assert.sameValue(JSON.stringify(new Number(5)), "17");

delete Number.prototype.valueOf;
assert.sameValue(JSON.stringify(new Number(5)), "null"); // isNaN(Number("[object Number]"))

delete Object.prototype.toString;
assert.throws(TypeError, function() {
  JSON.stringify(new Number(5));
}, "ToNumber failure, should throw TypeError");

delete Object.prototype.valueOf;
assert.throws(TypeError, function() {
  JSON.stringify(new Number(5));
}, "ToNumber failure, should throw TypeError");


redefine(Number.prototype, "toString", numToString);
redefine(Number.prototype, "valueOf", numValueOf);
redefine(Object.prototype, "toString", objToString);
redefine(Object.prototype, "valueOf", objValueOf);


redefine(String.prototype, "valueOf", function() { return 17; });
assert.sameValue(JSON.stringify(new String(5)), '"5"');

redefine(String.prototype, "toString", function() { return 42; });
assert.sameValue(JSON.stringify(new String(5)), '"42"');

delete String.prototype.toString;
assert.sameValue(JSON.stringify(new String(5)), '"[object String]"');

delete Object.prototype.toString;
assert.sameValue(JSON.stringify(new String(5)), '"17"');

delete String.prototype.valueOf;
assert.throws(TypeError, function() {
  JSON.stringify(new String(5));
}, "ToString failure, should throw TypeError");

delete Object.prototype.valueOf;
assert.throws(TypeError, function() {
  JSON.stringify(new String(5));
}, "ToString failure, should throw TypeError");
