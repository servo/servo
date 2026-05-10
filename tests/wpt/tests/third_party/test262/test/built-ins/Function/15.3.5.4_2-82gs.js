// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-82gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (non-strict function declaration called by
    strict Function.prototype.apply(someObject))
flags: [noStrict]
features: [caller]
---*/

function f() {
  return gNonStrict();
};
var o = {};
(function() {
  "use strict";
  f.apply(o);
})();


function gNonStrict() {
  return gNonStrict.caller;
}
