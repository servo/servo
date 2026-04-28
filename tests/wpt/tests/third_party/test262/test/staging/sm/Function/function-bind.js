/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [nativeFunctionMatcher.js]
flags:
  - noStrict
description: |
  Function.prototype.bind
info: bugzilla.mozilla.org/show_bug.cgi?id=429507
esid: pending
---*/

// ad-hoc testing

assert.sameValue(Function.prototype.hasOwnProperty("bind"), true);

var bind = Function.prototype.bind;
assert.sameValue(bind.length, 1);


var strictReturnThis = function() { "use strict"; return this; };

assert.sameValue(strictReturnThis.bind(undefined)(), undefined);
assert.sameValue(strictReturnThis.bind(null)(), null);

var obj = {};
assert.sameValue(strictReturnThis.bind(obj)(), obj);

assert.sameValue(strictReturnThis.bind(NaN)(), NaN);

assert.sameValue(strictReturnThis.bind(true)(), true);
assert.sameValue(strictReturnThis.bind(false)(), false);

assert.sameValue(strictReturnThis.bind("foopy")(), "foopy");


// rigorous, step-by-step testing

/*
 * 1. Let Target be the this value.
 * 2. If IsCallable(Target) is false, throw a TypeError exception.
 */
assert.throws(TypeError, function() { bind.call(null); });
assert.throws(TypeError, function() { bind.call(undefined); });
assert.throws(TypeError, function() { bind.call(NaN); });
assert.throws(TypeError, function() { bind.call(0); });
assert.throws(TypeError, function() { bind.call(-0); });
assert.throws(TypeError, function() { bind.call(17); });
assert.throws(TypeError, function() { bind.call(42); });
assert.throws(TypeError, function() { bind.call("foobar"); });
assert.throws(TypeError, function() { bind.call(true); });
assert.throws(TypeError, function() { bind.call(false); });
assert.throws(TypeError, function() { bind.call([]); });
assert.throws(TypeError, function() { bind.call({}); });


/*
 * 3. Let A be a new (possibly empty) internal list of all of the argument
 *    values provided after thisArg (arg1, arg2 etc), in order.
 * 4. Let F be a new native ECMAScript object .
 * 5. Set all the internal methods, except for [[Get]], of F as specified in
 *    8.12.
 * 6. Set the [[Get]] internal property of F as specified in 15.3.5.4.
 * 7. Set the [[TargetFunction]] internal property of F to Target.
 * 8. Set the [[BoundThis]] internal property of F to the value of thisArg.
 * 9. Set the [[BoundArgs]] internal property of F to A.
 */
// throughout


/* 10. Set the [[Class]] internal property of F to "Function". */
var toString = Object.prototype.toString;
assert.sameValue(toString.call(function(){}), "[object Function]");
assert.sameValue(toString.call(function a(){}), "[object Function]");
assert.sameValue(toString.call(function(a){}), "[object Function]");
assert.sameValue(toString.call(function a(b){}), "[object Function]");
assert.sameValue(toString.call(function(){}.bind()), "[object Function]");
assert.sameValue(toString.call(function a(){}.bind()), "[object Function]");
assert.sameValue(toString.call(function(a){}.bind()), "[object Function]");
assert.sameValue(toString.call(function a(b){}.bind()), "[object Function]");


/*
 * 11. Set the [[Prototype]] internal property of F to the standard built-in
 *     Function prototype object as specified in 15.3.3.1.
 */
assert.sameValue(Object.getPrototypeOf(bind.call(function(){})), Function.prototype);
assert.sameValue(Object.getPrototypeOf(bind.call(function a(){})), Function.prototype);
assert.sameValue(Object.getPrototypeOf(bind.call(function(a){})), Function.prototype);
assert.sameValue(Object.getPrototypeOf(bind.call(function a(b){})), Function.prototype);


/*
 * 12. Set the [[Call]] internal property of F as described in 15.3.4.5.1.
 */
var a = Array.bind(1, 2);
assert.sameValue(a().length, 2);
assert.sameValue(a(4).length, 2);
assert.sameValue(a(4, 8).length, 3);

function t() { return this; }
var bt = t.bind(t);
assert.sameValue(bt(), t);

function callee() { return arguments.callee; }
var call = callee.bind();
assert.sameValue(call(), callee);
assert.sameValue(new call(), callee);


/*
 * 13. Set the [[Construct]] internal property of F as described in 15.3.4.5.2.
 */
function Point(x, y)
{
  this.x = x;
  this.y = y;
}
var YAxisPoint = Point.bind(null, 0)

assert.sameValue(YAxisPoint.hasOwnProperty("prototype"), false);
var p = new YAxisPoint(5);
assert.sameValue(p.x, 0);
assert.sameValue(p.y, 5);
assert.sameValue(p instanceof Point, true);
assert.sameValue(p instanceof YAxisPoint, true);
assert.sameValue(Object.prototype.toString.call(YAxisPoint), "[object Function]");
assert.sameValue(YAxisPoint.length, 1);


/*
 * 14. Set the [[HasInstance]] internal property of F as described in
 *     15.3.4.5.3.
 */
function JoinArguments()
{
  this.args = Array.prototype.join.call(arguments, ", ");
}

var Join1 = JoinArguments.bind(null, 1);
var Join2 = Join1.bind(null, 2);
var Join3 = Join2.bind(null, 3);
var Join4 = Join3.bind(null, 4);
var Join5 = Join4.bind(null, 5);
var Join6 = Join5.bind(null, 6);

var r = new Join6(7);
assert.sameValue(r instanceof Join6, true);
assert.sameValue(r instanceof Join5, true);
assert.sameValue(r instanceof Join4, true);
assert.sameValue(r instanceof Join3, true);
assert.sameValue(r instanceof Join2, true);
assert.sameValue(r instanceof Join1, true);
assert.sameValue(r instanceof JoinArguments, true);
assert.sameValue(r.args, "1, 2, 3, 4, 5, 6, 7");


/*
 * 15. If the [[Class]] internal property of Target is "Function", then
 *   a. Let L be the length property of Target minus the length of A.
 *   b. Set the length own property of F to either 0 or L, whichever is larger.
 * 16. Else set the length own property of F to 0.
 */
function none() { return arguments.length; }
assert.sameValue(none.bind(1, 2)(3, 4), 3);
assert.sameValue(none.bind(1, 2)(), 1);
assert.sameValue(none.bind(1)(2, 3), 2);
assert.sameValue(none.bind().length, 0);
assert.sameValue(none.bind(null).length, 0);
assert.sameValue(none.bind(null, 1).length, 0);
assert.sameValue(none.bind(null, 1, 2).length, 0);

function one(a) { }
assert.sameValue(one.bind().length, 1);
assert.sameValue(one.bind(null).length, 1);
assert.sameValue(one.bind(null, 1).length, 0);
assert.sameValue(one.bind(null, 1, 2).length, 0);

// retch
var br = Object.create(null, { length: { value: 0 } });
assert.throws(TypeError, function() {
  bind.call(/a/g, /a/g, "aaaa");
});
assert.sameValue(br.length, 0);


/*
 * 17. Set the attributes of the length own property of F to the values
 *     specified in 15.3.5.1.
 */
var len1Desc =
  Object.getOwnPropertyDescriptor(function(a, b, c){}.bind(), "length");
assert.sameValue(len1Desc.value, 3);
assert.sameValue(len1Desc.writable, false);
assert.sameValue(len1Desc.enumerable, false);
assert.sameValue(len1Desc.configurable, true);

var len2Desc =
  Object.getOwnPropertyDescriptor(function(a, b, c){}.bind(null, 2), "length");
assert.sameValue(len2Desc.value, 2);
assert.sameValue(len2Desc.writable, false);
assert.sameValue(len2Desc.enumerable, false);
assert.sameValue(len2Desc.configurable, true);


/*
 * 18. Set the [[Extensible]] internal property of F to true.
 */
var bound = (function() { }).bind();

var isExtensible = Object.isExtensible || function() { return true; };
assert.sameValue(isExtensible(bound), true);

bound.foo = 17;
var fooDesc = Object.getOwnPropertyDescriptor(bound, "foo");
assert.sameValue(fooDesc.value, 17);
assert.sameValue(fooDesc.writable, true);
assert.sameValue(fooDesc.enumerable, true);
assert.sameValue(fooDesc.configurable, true);


/*
 * Steps 19-21 are removed from ES6, instead implemented through "arguments" and
 * "caller" accessors on Function.prototype.  So no own properties, but do check
 * for the same observable behavior (modulo where the accessors live).
 */
function strict() { "use strict"; }
function nonstrict() {}

function testBound(fun)
{
  var boundf = fun.bind();

  assert.sameValue(Object.getOwnPropertyDescriptor(boundf, "arguments"), undefined,
           "should be no arguments property");
  assert.sameValue(Object.getOwnPropertyDescriptor(boundf, "caller"), undefined,
           "should be no caller property");

  assert.throws(TypeError, function() { return boundf.arguments; });
  assert.throws(TypeError, function() { return boundf.caller; });
}

testBound(strict);
testBound(nonstrict);

assertNativeFunction(function unbound(){"body"}.bind());

/* 22. Return F. */
var passim = function p(){}.bind(1);
assert.sameValue(typeof passim, "function");
