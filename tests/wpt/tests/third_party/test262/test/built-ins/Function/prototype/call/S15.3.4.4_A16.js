// Copyright 2011 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: If IsCallable(func) is false, then throw a TypeError exception.
es5id: 15.3.4.4_A16
description: >
    A RegExp is not a function, but it may be callable. Iff it is,
    it's typeof should be 'function', in which case call should accept
    it as a valid this value.
---*/

var re = (/x/);
if (typeof re === 'function') {
  Function.prototype.call.call(re, undefined, 'x');
} else {
  try {
    Function.prototype.bind.call(re, undefined);
    throw new Test262Error('#1: If IsCallable(func) is false, ' +
      'then (bind should) throw a TypeError exception');
  } catch (e) {
    assert(e instanceof TypeError, 'The result of evaluating (e instanceof TypeError) is expected to be true');
  }
}
