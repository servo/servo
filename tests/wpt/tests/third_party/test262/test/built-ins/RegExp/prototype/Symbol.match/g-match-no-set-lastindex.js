// Copyright (C) 2015 Mike Pennisi. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The `lastIndex` is not set after a non-zero-width match
es6id: 21.2.5.6
info: |
    7. If global is false, then
       [...]
    8. Else global is true,
       [...]
       g. Repeat,
          i. Let result be RegExpExec(rx, S).
          [...]
          iv. Else result is not null,
              1. Let matchStr be ToString(Get(result, "0")).
              [...]
              5. If matchStr is the empty String, then
                 [...]
                 d. Let setStatus be Set(rx, "lastIndex", nextIndex, true).
                 e. ReturnIfAbrupt(setStatus).
features: [Symbol.match]
---*/

var exec = function() {
  var thisMatch = nextMatch;
  if (thisMatch === null) {
    return null;
  }
  nextMatch = null;
  return {
    get 0() {
      Object.defineProperty(r, 'lastIndex', { writable: false });
      return thisMatch;
    }
  };
};
var r, nextMatch;

r = /./g;
r.exec = exec;
nextMatch = 'a non-empty string';
r[Symbol.match]('');
