/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  f.arguments must trigger an arguments object in non-strict mode functions
info: bugzilla.mozilla.org/show_bug.cgi?id=721322
esid: pending
---*/

var obj =
  {
    test: function()
    {
      var args = obj.test.arguments;
      assert.sameValue(args !== null, true);
      assert.sameValue(args[0], 5);
      assert.sameValue(args[1], undefined);
      assert.sameValue(args.length, 2);
    }
  };
obj.test(5, undefined);

var sobj =
  {
    test: function()
    {
     "use strict";

      assert.throws(TypeError, function() {
        sobj.test.arguments;
      }, "access to arguments property of strict mode function");
    }
  };
sobj.test(5, undefined);
