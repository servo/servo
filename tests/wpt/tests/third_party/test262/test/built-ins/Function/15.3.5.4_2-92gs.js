// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-92gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (non-strict function declaration called by
    strict Function.prototype.bind(someObject)())
flags: [noStrict]
features: [caller]
---*/

function f() {
  return gNonStrict();
};
var o = {};
(function() {
  "use strict";
  f.bind(o)();
})();


function gNonStrict() {
  return gNonStrict.caller;
}
