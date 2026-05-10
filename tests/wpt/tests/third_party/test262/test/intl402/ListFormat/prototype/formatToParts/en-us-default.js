// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.formatToParts
description: >
    Checks the behavior of Intl.ListFormat.prototype.formatToParts() in English.
features: [Intl.ListFormat]
locale: [en-US]
---*/

function verifyFormatParts(actual, expected, message) {
  assert.sameValue(actual.length, expected.length, `${message}: length`);

  for (let i = 0; i < actual.length; ++i) {
    assert.sameValue(actual[i].type, expected[i].type, `${message}: parts[${i}].type`);
    assert.sameValue(actual[i].value, expected[i].value, `${message}: parts[${i}].value`);
  }
}

function CustomIterator(array) {
  this.i = 0;
  this.array = array;
}

CustomIterator.prototype[Symbol.iterator] = function() {
  return this;
}

CustomIterator.prototype.next = function() {
  if (this.i >= this.array.length) {
    return {
      "done": true,
    };
  }

  return {
    "value": this.array[this.i++],
    "done": false,
  };
}

const transforms = [
  a => a,
  a => a[Symbol.iterator](),
  a => new CustomIterator(a),
];

const lf = new Intl.ListFormat("en-US");

assert.sameValue(typeof lf.formatToParts, "function", "formatToParts should be supported");

for (const f of transforms) {
  verifyFormatParts(lf.formatToParts(f([])), []);
  verifyFormatParts(lf.formatToParts(f(["foo"])), [
    { "type": "element", "value": "foo" },
  ]);
  verifyFormatParts(lf.formatToParts(f(["foo", "bar"])), [
    { "type": "element", "value": "foo" },
    { "type": "literal", "value": " and " },
    { "type": "element", "value": "bar" },
  ]);
  verifyFormatParts(lf.formatToParts(f(["foo", "bar", "baz"])), [
    { "type": "element", "value": "foo" },
    { "type": "literal", "value": ", " },
    { "type": "element", "value": "bar" },
    { "type": "literal", "value": ", and " },
    { "type": "element", "value": "baz" },
  ]);
  verifyFormatParts(lf.formatToParts(f(["foo", "bar", "baz", "quux"])), [
    { "type": "element", "value": "foo" },
    { "type": "literal", "value": ", " },
    { "type": "element", "value": "bar" },
    { "type": "literal", "value": ", " },
    { "type": "element", "value": "baz" },
    { "type": "literal", "value": ", and " },
    { "type": "element", "value": "quux" },
  ]);
}

verifyFormatParts(lf.formatToParts("foo"), [
  { "type": "element", "value": "f" },
  { "type": "literal", "value": ", " },
  { "type": "element", "value": "o" },
  { "type": "literal", "value": ", and " },
  { "type": "element", "value": "o" },
]);
