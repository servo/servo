// Copyright (C) 2017 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-generator-function-definitions-runtime-semantics-evaluation
description: >
    If <iterator>.return is an object emulating `undefined` (e.g. `document.all`
    in browsers), it shouldn't be treated as if it were actually `undefined` by
    the yield* operator.
features: [generators, IsHTMLDDA]
---*/

var IsHTMLDDA = $262.IsHTMLDDA;
var iter = {
  [Symbol.iterator]() { return this; },
  next() { return {}; },
  return: IsHTMLDDA,
};

var outer = (function*() { yield* iter; })();

outer.next();

assert.throws(TypeError, function() {
  // `IsHTMLDDA` is called here with `iter` as `this` and `emptyString` as the
  // sole argument, and it's specified to return `null` under these conditions.
  // As `iter`'s iteration isn't ending because of a throw, the iteration
  // protocol will then throw a `TypeError` because `null` isn't an object.
  var emptyString = "";
  outer.return(emptyString);
});
