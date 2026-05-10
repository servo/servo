// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 12.10-0-1
description: >
    with does not change declaration scope - vars in with are visible
    outside
flags: [noStrict]
---*/

  var o = {};
  var f = function () {
	/* capture foo binding before executing with */
	return foo;
      }

  with (o) {
    var foo = "12.10-0-1";
  }

assert.sameValue(f(), "12.10-0-1", 'f()');
