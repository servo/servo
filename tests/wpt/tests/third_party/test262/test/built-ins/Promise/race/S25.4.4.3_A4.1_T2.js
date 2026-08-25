// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
es6id: S25.4.4.3_A4.1_T2
author: Sam Mikes
description: Promise.race rejects if IteratorStep throws
features: [Symbol.iterator]
flags: [async]
---*/

var iterThrows = {};
Object.defineProperty(iterThrows, Symbol.iterator, {
  get: function() {
    return {
      next: function() {
        var v = {};
        Object.defineProperty(v, 'value', {
          get: function() {
            throw new Error("abrupt completion");
          }
        });
        return v;
      }
    };
  }
});

Promise.race(iterThrows).then(function() {
  throw new Test262Error('Promise unexpectedly fulfilled: Promise.race(iterThrows) should throw TypeError');
}, function(err) {
  assert(!!(err instanceof TypeError), 'The value of !!(err instanceof TypeError) is expected to be true');
}).then($DONE, $DONE);
