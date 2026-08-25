/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Object.defineProperties(O, Properties)
info: bugzilla.mozilla.org/show_bug.cgi?id=430133
esid: pending
---*/

assert.sameValue("defineProperties" in Object, true);
assert.sameValue(Object.defineProperties.length, 2);

var o, props, desc, passed;

o = {};
props =
  {
    a: { value: 17, enumerable: true, configurable: true, writable: true },
    b: { value: 42, enumerable: false, configurable: false, writable: false }
  };
Object.defineProperties(o, props);
assert.sameValue("a" in o, true);
assert.sameValue("b" in o, true);
desc = Object.getOwnPropertyDescriptor(o, "a");
assert.sameValue(desc.value, 17);
assert.sameValue(desc.enumerable, true);
assert.sameValue(desc.configurable, true);
assert.sameValue(desc.writable, true);
desc = Object.getOwnPropertyDescriptor(o, "b");
assert.sameValue(desc.value, 42);
assert.sameValue(desc.enumerable, false);
assert.sameValue(desc.configurable, false);
assert.sameValue(desc.writable, false);

props =
  {
    c: { value: NaN, enumerable: false, configurable: true, writable: true },
    b: { value: 44 }
  };

assert.throws(TypeError, function() {
  Object.defineProperties(o, props);
});
assert.sameValue("c" in o, true, "new property added");
assert.sameValue(o.b, 42, "old property value preserved");

function Properties() { }
Properties.prototype = { b: { value: 42, enumerable: true } };
props = new Properties();
Object.defineProperty(props, "a", { enumerable: false });
o = {};
Object.defineProperties(o, props);
assert.sameValue("a" in o, false);
assert.sameValue(Object.getOwnPropertyDescriptor(o, "a"), undefined,
         "Object.defineProperties(O, Properties) should only use enumerable " +
         "properties on Properties");
assert.sameValue("b" in o, false);
assert.sameValue(Object.getOwnPropertyDescriptor(o, "b"), undefined,
         "Object.defineProperties(O, Properties) should only use enumerable " +
         "properties directly on Properties");

Number.prototype.foo = { value: 17, enumerable: true };
Boolean.prototype.bar = { value: 8675309, enumerable: true };
String.prototype.quux = { value: "Are you HAPPY yet?", enumerable: true };
o = {};
Object.defineProperties(o, 5); // ToObject only throws for null/undefined
assert.sameValue("foo" in o, false, "foo is not an enumerable own property");
Object.defineProperties(o, false);
assert.sameValue("bar" in o, false, "bar is not an enumerable own property");
Object.defineProperties(o, "");
assert.sameValue("quux" in o, false, "quux is not an enumerable own property");

assert.throws(TypeError, function() {
  Object.defineProperties(o, "1");
}, "should throw on Properties == '1' due to '1'[0] not being a property descriptor");

assert.throws(TypeError, function() {
  Object.defineProperties(o, null);
}, "should throw on Properties == null");

assert.throws(TypeError, function() {
  Object.defineProperties(o, undefined);
}, "should throw on Properties == undefined");
