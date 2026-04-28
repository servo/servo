// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function assertIsObject(x)
{
  assert.sameValue(typeof x, "object");
  assert.sameValue(x instanceof Object, true);
}

function assertIsArray(x)
{
  assertIsObject(x);
  assert.sameValue(Array.isArray(x), true);
  assert.sameValue(Object.getPrototypeOf(x), Array.prototype);
  assert.sameValue(x instanceof Array, true);
  assert.sameValue(x.constructor, Array);
}

var x;
var props;

// empty object
x = JSON.parse("{}");
assertIsObject(x);
assert.sameValue(Object.getOwnPropertyNames(x).length, 0);

// empty array
x = JSON.parse("[]");
assertIsArray(x);
assert.sameValue(x.length, 0);

// one element array
x = JSON.parse("[[]]");
assertIsArray(x);
assert.sameValue(x.length, 1);
assertIsArray(x[0]);
assert.sameValue(x[0].length, 0);

// multiple arrays
x = JSON.parse("[[],[],[]]");
assertIsArray(x);
assert.sameValue(x.length, 3);
assertIsArray(x[0]);
assert.sameValue(x[0].length, 0);
assertIsArray(x[1]);
assert.sameValue(x[1].length, 0);
assertIsArray(x[2]);
assert.sameValue(x[2].length, 0);

// array key/value
x = JSON.parse('{"foo":[]}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x);
assert.sameValue(props.length, 1);
assert.sameValue(props[0], "foo");
assertIsArray(x.foo);
assert.sameValue(x.foo.length, 0);

x = JSON.parse('{"foo":[], "bar":[]}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x).sort();
assert.sameValue(props.length, 2);
assert.sameValue(props[0], "bar");
assert.sameValue(props[1], "foo");
assertIsArray(x.foo);
assert.sameValue(x.foo.length, 0);
assertIsArray(x.bar);
assert.sameValue(x.bar.length, 0);

// nesting
x = JSON.parse('{"foo":[{}]}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x);
assert.sameValue(props.length, 1);
assert.sameValue(props[0], "foo");
assertIsArray(x.foo);
assert.sameValue(x.foo.length, 1);
assertIsObject(x.foo[0]);
assert.sameValue(Object.getOwnPropertyNames(x.foo[0]).length, 0);

x = JSON.parse('{"foo":[{"foo":[{"foo":{}}]}]}');
assertIsObject(x.foo[0].foo[0].foo);

x = JSON.parse('{"foo":[{"foo":[{"foo":[]}]}]}');
assertIsArray(x.foo[0].foo[0].foo);

// strings
x = JSON.parse('{"foo":"bar"}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x);
assert.sameValue(props.length, 1);
assert.sameValue(props[0], "foo");
assert.sameValue(x.foo, "bar");

x = JSON.parse('["foo", "bar", "baz"]');
assertIsArray(x);
assert.sameValue(x.length, 3);
assert.sameValue(x[0], "foo");
assert.sameValue(x[1], "bar");
assert.sameValue(x[2], "baz");

// numbers
x = JSON.parse('{"foo":5.5, "bar":5}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x).sort();
assert.sameValue(props.length, 2);
assert.sameValue(props[0], "bar");
assert.sameValue(props[1], "foo");
assert.sameValue(x.foo, 5.5);
assert.sameValue(x.bar, 5);

// keywords
x = JSON.parse('{"foo": true, "bar":false, "baz":null}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x).sort();
assert.sameValue(props.length, 3);
assert.sameValue(props[0], "bar");
assert.sameValue(props[1], "baz");
assert.sameValue(props[2], "foo");
assert.sameValue(x.foo, true);
assert.sameValue(x.bar, false);
assert.sameValue(x.baz, null);

// short escapes
x = JSON.parse('{"foo": "\\"", "bar":"\\\\", "baz":"\\b","qux":"\\f", "quux":"\\n", "quuux":"\\r","quuuux":"\\t"}');
props = Object.getOwnPropertyNames(x).sort();
assert.sameValue(props.length, 7);
assert.sameValue(props[0], "bar");
assert.sameValue(props[1], "baz");
assert.sameValue(props[2], "foo");
assert.sameValue(props[3], "quuuux");
assert.sameValue(props[4], "quuux");
assert.sameValue(props[5], "quux");
assert.sameValue(props[6], "qux");
assert.sameValue(x.foo, '"');
assert.sameValue(x.bar, '\\');
assert.sameValue(x.baz, '\b');
assert.sameValue(x.qux, '\f');
assert.sameValue(x.quux, "\n");
assert.sameValue(x.quuux, "\r");
assert.sameValue(x.quuuux, "\t");

// unicode escape
x = JSON.parse('{"foo":"hmm\\u006dmm"}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x);
assert.sameValue(props.length, 1);
assert.sameValue(props[0], "foo");
assert.sameValue("hmm\u006dmm", x.foo);

x = JSON.parse('{"hmm\\u006dmm":"foo"}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x);
assert.sameValue(props.length, 1);
assert.sameValue(props[0], "hmmmmm");
assert.sameValue(x.hmm\u006dmm, "foo");

// miscellaneous
x = JSON.parse('{"JSON Test Pattern pass3": {"The outermost value": "must be an object or array.","In this test": "It is an object." }}');
assertIsObject(x);
props = Object.getOwnPropertyNames(x);
assert.sameValue(props.length, 1);
assert.sameValue(props[0], "JSON Test Pattern pass3");
assertIsObject(x["JSON Test Pattern pass3"]);
props = Object.getOwnPropertyNames(x["JSON Test Pattern pass3"]).sort();
assert.sameValue(props.length, 2);
assert.sameValue(props[0], "In this test");
assert.sameValue(props[1], "The outermost value");
assert.sameValue(x["JSON Test Pattern pass3"]["The outermost value"],
         "must be an object or array.");
assert.sameValue(x["JSON Test Pattern pass3"]["In this test"], "It is an object.");
