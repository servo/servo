// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Mapped arguments object with non-configurable property
description: >
    Mapping works when property is non-configurable, arguments property
    was not deleted. [[Delete]] operations throws TypeError if called
    from strict-mode code. Arguments property is changed with
    [[DefineOwnProperty]].
flags: [noStrict]
---*/

function argumentsAndStrictDeleteDefineOwnProperty(a) {
  Object.defineProperty(arguments, "0", {configurable: false});

  // Precondition: Delete is unsuccessful and doesn't affect mapping.
  var args = arguments;
  assert.throws(TypeError, function() { "use strict"; delete args[0]; });
  assert.sameValue(a, 1);
  assert.sameValue(arguments[0], 1);

  Object.defineProperty(arguments, "0", {value: 2});
  assert.sameValue(a, 2);
  assert.sameValue(arguments[0], 2);
}
argumentsAndStrictDeleteDefineOwnProperty(1);
