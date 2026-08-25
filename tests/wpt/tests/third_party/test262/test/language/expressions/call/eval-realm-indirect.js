// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-function-calls-runtime-semantics-evaluation
es6id: 12.3.4.1
description: >
    An eval function from another realm is not a candidate for direct eval
info: |
    [...]
    3. If Type(ref) is Reference and IsPropertyReference(ref) is false and GetReferencedName(ref) is "eval", then
       a. If SameValue(func, %eval%) is true, then
          [...]
flags: [noStrict]
features: [cross-realm]
---*/

var x = 'outside';
var result;

(function() {
  var eval = $262.createRealm().global.eval;

  eval('var x = "inside";');

  result = x;
}());

assert.sameValue(result, 'outside');
