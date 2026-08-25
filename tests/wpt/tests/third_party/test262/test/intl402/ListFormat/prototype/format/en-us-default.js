// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat.prototype.format
description: >
    Checks the behavior of Intl.ListFormat.prototype.format() in English.
features: [Intl.ListFormat]
locale: [en-US]
---*/

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

assert.sameValue(typeof lf.format, "function", "format should be supported");

for (const f of transforms) {
  assert.sameValue(lf.format(f([])), "");
  assert.sameValue(lf.format(f(["foo"])), "foo");
  assert.sameValue(lf.format(f(["foo", "bar"])), "foo and bar");
  assert.sameValue(lf.format(f(["foo", "bar", "baz"])), "foo, bar, and baz");
  assert.sameValue(lf.format(f(["foo", "bar", "baz", "quux"])), "foo, bar, baz, and quux");
}

assert.sameValue(lf.format("foo"), "f, o, and o");
