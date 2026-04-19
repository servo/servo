// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function bogusConstruct(target) { return 4; }
function bogusConstructUndefined(target) { }

var handler = { construct: bogusConstruct }

function callable() {}

var p = new Proxy(callable, handler);

assert.throws(TypeError, function () { new p(); },
                       "[[Construct must throw if an object is not returned.");

handler.construct = bogusConstructUndefined;
assert.throws(TypeError, function () { new p(); },
                       "[[Construct must throw if an object is not returned.");

