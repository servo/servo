/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Implement __define[GS]etter__ using Object.defineProperty
info: bugzilla.mozilla.org/show_bug.cgi?id=715821
esid: pending
---*/

function s(desc)
{
  if (typeof desc === "undefined")
    return "<undefined>";
  assert.sameValue(typeof desc, "object");
  assert.sameValue(desc !== null, true);

  var str = "<enumerable: <" + desc.enumerable + ">, " +
            " configurable: <" + desc.configurable + ">,";

  if (desc.hasOwnProperty("value"))
  {
    return str +
           " value: <" + desc.value + ">," +
           " writable: <" + desc.writable + ">>";
  }

  return str +
         " get: <" + desc.get + ">," +
         " set: <" + desc.set + ">>";
}

function checkField(field, desc, expected)
{
  var present = desc.hasOwnProperty(field);
  assert.sameValue(present, expected.hasOwnProperty(field),
           field + " presence mismatch (got " + s(desc) + ", expected " + s(expected) + ")");
  if (present)
  {
    assert.sameValue(desc[field], expected[field],
             field + " value mismatch (got " + s(desc) + ", expected " + s(expected) + ")");
  }
}

function check(obj, prop, expected)
{
  var desc = Object.getOwnPropertyDescriptor(obj, prop);
  assert.sameValue(typeof desc, typeof expected,
           "type mismatch (got " + s(desc) + ", expected " + s(expected) + ")");

  assert.sameValue(desc.hasOwnProperty("get"), desc.hasOwnProperty("set"),
           "bad descriptor: " + s(desc));
  assert.sameValue(desc.hasOwnProperty("value"), desc.hasOwnProperty("writable"),
           "bad descriptor: " + s(desc));

  assert.sameValue(desc.hasOwnProperty("get"), !desc.hasOwnProperty("value"),
           "bad descriptor: " + s(desc));

  checkField("get", desc, expected);
  checkField("set", desc, expected);
  checkField("value", desc, expected);
  checkField("writable", desc, expected);
  checkField("enumerable", desc, expected);
  checkField("configurable", desc, expected);
}

/**************
 * BEGIN TEST *
 **************/

// Adding a new getter, overwriting an existing one

function g1() { }
var gobj = {};
gobj.__defineGetter__("foo", g1);
check(gobj, "foo", { get: g1, set: undefined, enumerable: true, configurable: true });

function g2() { }
gobj.__defineGetter__("foo", g2);
check(gobj, "foo", { get: g2, set: undefined, enumerable: true, configurable: true });

/******************************************************************************/

// Adding a new setter, overwriting an existing one

function s1() { }
var sobj = {};
sobj.__defineSetter__("bar", s1);
check(sobj, "bar", { get: undefined, set: s1, enumerable: true, configurable: true });

function s2() { }
sobj.__defineSetter__("bar", s2);
check(sobj, "bar", { get: undefined, set: s2, enumerable: true, configurable: true });

/******************************************************************************/

// Adding a new getter, then adding a setter
// Changing an existing accessor's enumerability, then "null"-changing the accessor
// Changing an accessor's configurability, then "null"-changing and real-changing the accessor

function g3() { }
var gsobj = {};
gsobj.__defineGetter__("baz", g3);
check(gsobj, "baz", { get: g3, set: undefined, enumerable: true, configurable: true });

function s3() { }
gsobj.__defineSetter__("baz", s3);
check(gsobj, "baz", { get: g3, set: s3, enumerable: true, configurable: true });

Object.defineProperty(gsobj, "baz", { enumerable: false });
check(gsobj, "baz", { get: g3, set: s3, enumerable: false, configurable: true });

gsobj.__defineGetter__("baz", g3);
check(gsobj, "baz", { get: g3, set: s3, enumerable: true, configurable: true });

Object.defineProperty(gsobj, "baz", { enumerable: false });
check(gsobj, "baz", { get: g3, set: s3, enumerable: false, configurable: true });

gsobj.__defineSetter__("baz", s3);
check(gsobj, "baz", { get: g3, set: s3, enumerable: true, configurable: true });

Object.defineProperty(gsobj, "baz", { configurable: false });
assert.throws(TypeError, function() { gsobj.__defineSetter__("baz", s2); });
assert.throws(TypeError, function() { gsobj.__defineSetter__("baz", s3); });
check(gsobj, "baz", { get: g3, set: s3, enumerable: true, configurable: false });

/******************************************************************************/

// Adding a new setter, then adding a getter
// Changing an existing accessor's enumerability, then "null"-changing the accessor
// Changing an accessor's configurability, then "null"-changing and real-changing the accessor

function s4() { }
var sgobj = {};
sgobj.__defineSetter__("baz", s4);
check(sgobj, "baz", { get: undefined, set: s4, enumerable: true, configurable: true });

function g4() { }
sgobj.__defineGetter__("baz", g4);
check(sgobj, "baz", { get: g4, set: s4, enumerable: true, configurable: true });

Object.defineProperty(sgobj, "baz", { enumerable: false });
check(sgobj, "baz", { get: g4, set: s4, enumerable: false, configurable: true });

sgobj.__defineSetter__("baz", s4);
check(sgobj, "baz", { get: g4, set: s4, enumerable: true, configurable: true });

Object.defineProperty(sgobj, "baz", { enumerable: false });
check(sgobj, "baz", { get: g4, set: s4, enumerable: false, configurable: true });

sgobj.__defineSetter__("baz", s4);
check(sgobj, "baz", { get: g4, set: s4, enumerable: true, configurable: true });

Object.defineProperty(sgobj, "baz", { configurable: false });
assert.throws(TypeError, function() { sgobj.__defineGetter__("baz", g3); });
assert.throws(TypeError, function() { sgobj.__defineSetter__("baz", s4); });
check(sgobj, "baz", { get: g4, set: s4, enumerable: true, configurable: false });

/******************************************************************************/

// Adding a getter over a writable data property

function g5() { }
var gover = { quux: 17 };
check(gover, "quux", { value: 17, writable: true, enumerable: true, configurable: true });

gover.__defineGetter__("quux", g5);
check(gover, "quux", { get: g5, set: undefined, enumerable: true, configurable: true });

/******************************************************************************/

// Adding a setter over a writable data property

function s5() { }
var sover = { quux: 17 };
check(sover, "quux", { value: 17, writable: true, enumerable: true, configurable: true });

sover.__defineSetter__("quux", s5);
check(sover, "quux", { get: undefined, set: s5, enumerable: true, configurable: true });

/******************************************************************************/

// Adding a getter over a non-writable data property

function g6() { }
var gnover = { eit: 17 };
check(gnover, "eit", { value: 17, writable: true, enumerable: true, configurable: true });
Object.defineProperty(gnover, "eit", { writable: false });
check(gnover, "eit", { value: 17, writable: false, enumerable: true, configurable: true });

gnover.__defineGetter__("eit", g6);
check(gnover, "eit", { get: g6, set: undefined, enumerable: true, configurable: true });

/******************************************************************************/

// Adding a setter over a non-writable data property

function s6() { }
var snover = { eit: 17 };
check(snover, "eit", { value: 17, writable: true, enumerable: true, configurable: true });
Object.defineProperty(snover, "eit", { writable: false });
check(snover, "eit", { value: 17, writable: false, enumerable: true, configurable: true });

snover.__defineSetter__("eit", s6);
check(snover, "eit", { get: undefined, set: s6, enumerable: true, configurable: true });

/******************************************************************************/

// Adding a getter over a non-configurable, writable data property

function g7() { }
var gncover = { moo: 17 };
check(gncover, "moo", { value: 17, writable: true, enumerable: true, configurable: true });
Object.defineProperty(gncover, "moo", { configurable: false });
check(gncover, "moo", { value: 17, writable: true, enumerable: true, configurable: false });

assert.throws(TypeError, function() { gncover.__defineGetter__("moo", g7); });
check(gncover, "moo", { value: 17, writable: true, enumerable: true, configurable: false });

/******************************************************************************/

// Adding a setter over a non-configurable, writable data property

function s7() { }
var sncover = { moo: 17 };
check(sncover, "moo", { value: 17, writable: true, enumerable: true, configurable: true });
Object.defineProperty(sncover, "moo", { configurable: false });
check(sncover, "moo", { value: 17, writable: true, enumerable: true, configurable: false });

assert.throws(TypeError, function() { sncover.__defineSetter__("moo", s7); });
check(sncover, "moo", { value: 17, writable: true, enumerable: true, configurable: false });

/******************************************************************************/

// Adding a getter over a non-configurable, non-writable data property

function g8() { }
var gncwover = { fwoosh: 17 };
check(gncwover, "fwoosh", { value: 17, writable: true, enumerable: true, configurable: true });
Object.defineProperty(gncwover, "fwoosh", { writable: false, configurable: false });
check(gncwover, "fwoosh", { value: 17, writable: false, enumerable: true, configurable: false });

assert.throws(TypeError, function() { gncwover.__defineGetter__("fwoosh", g7); });
check(gncwover, "fwoosh", { value: 17, writable: false, enumerable: true, configurable: false });

/******************************************************************************/

// Adding a setter over a non-configurable, non-writable data property

function s8() { }
var sncwover = { fwoosh: 17 };
check(sncwover, "fwoosh", { value: 17, writable: true, enumerable: true, configurable: true });
Object.defineProperty(sncwover, "fwoosh", { writable: false, configurable: false });
check(sncwover, "fwoosh", { value: 17, writable: false, enumerable: true, configurable: false });

assert.throws(TypeError, function() { sncwover.__defineSetter__("fwoosh", s7); });
check(sncwover, "fwoosh", { value: 17, writable: false, enumerable: true, configurable: false });
