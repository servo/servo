// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.5.4_2-85gs
description: >
    Strict mode - checking access to strict function caller from
    non-strict function (non-strict function declaration called by
    strict Function.prototype.call(null))
flags: [noStrict]
features: [caller]
---*/

function f() {
  return gNonStrict();
};
(function() {
  "use strict";
  f.call(null);
})();


function gNonStrict() {
  return gNonStrict.caller;
}
