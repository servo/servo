// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Trace RegExp.prototype[@@split] behavior.
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

var n;
var log;
var target;
var flags;
var expectedFlags;

var execResult;
var lastIndexResult;
var lastIndexExpected;

var arraySetterObserved = false;
function startObserve() {
  for (var i = 0; i < 10; i++) {
    Object.defineProperty(Array.prototype, i, {
      set: function(v) {
        arraySetterObserved = true;
      },
      configurable: true,
    });
  }
}
function stopObserve() {
  for (var i = 0; i < 10; i++)
    delete Array.prototype[i]
}

startObserve();

function P(A) {
  return new Proxy(A, {
    get(that, name) {
      log += "get:result[" + name + "],";
      return that[name];
    }
  });
}

var myRegExp = {
  get constructor() {
    log += "get:constructor,";
    return {
      get [Symbol.species]() {
        log += "get:species,";
        return function(pattern, flags) {
          assert.sameValue(pattern, myRegExp);
          assert.sameValue(flags, expectedFlags);
          log += "call:constructor,";
          return {
            get lastIndex() {
              log += "get:lastIndex,";
              return lastIndexResult[n];
            },
            set lastIndex(v) {
              log += "set:lastIndex,";
              assert.sameValue(v, lastIndexExpected[n]);
            },
            get flags() {
              log += "get:flags,";
              return flags;
            },
            get exec() {
              log += "get:exec,";
              return function(S) {
                log += "call:exec,";
                assert.sameValue(S, target);
                return execResult[n++];
              };
            },
          };
        };
      }
    };
  },
  get flags() {
    log += "get:flags,";
    return flags;
  },
};

function reset() {
  n = 0;
  log = "";
  target = "abcde";
  flags = "";
  expectedFlags = "y";
  arraySetterObserved = false;
}

// Trace match and no match.
reset();
execResult        = [    null, P(["b"]), null, P(["d"]), null ];
lastIndexResult   = [ ,  ,     2,        ,     4,        ,    ];
lastIndexExpected = [ 0, 1,    2,        3,    4,             ];
var ret = RegExp.prototype[Symbol.split].call(myRegExp, target);
assert.sameValue(arraySetterObserved, false);
assert.sameValue(JSON.stringify(ret), `["a","c","e"]`);
assert.sameValue(log,
         "get:constructor," +
         "get:species," +
         "get:flags," +
         "call:constructor," +
         "set:lastIndex,get:exec,call:exec," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "get:result[length]," +
         "set:lastIndex,get:exec,call:exec," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "get:result[length]," +
         "set:lastIndex,get:exec,call:exec,");

// Trace non-empty flags, empty target, no match.
reset();
flags = "iy";
expectedFlags = "iy";
target = "";
execResult        = [ null ];
lastIndexResult   = [];
lastIndexExpected = [];
ret = RegExp.prototype[Symbol.split].call(myRegExp, target);
assert.sameValue(arraySetterObserved, false);
assert.sameValue(JSON.stringify(ret), `[""]`);
assert.sameValue(log,
         "get:constructor," +
         "get:species," +
         "get:flags," +
         "call:constructor," +
         "get:exec,call:exec,");

// Trace empty target, match.
reset();
target = "";
execResult        = [ P([""]) ];
lastIndexResult   = [];
lastIndexExpected = [];
ret = RegExp.prototype[Symbol.split].call(myRegExp, target);
assert.sameValue(arraySetterObserved, false);
assert.sameValue(JSON.stringify(ret), `[]`);
assert.sameValue(log,
         "get:constructor," +
         "get:species," +
         "get:flags," +
         "call:constructor," +
         "get:exec,call:exec,");

// Trace captures.
reset();
target = "abc";
execResult        = [    null, P(["b", "X", "YZ"]), null ];
lastIndexResult   = [ ,  ,     2,                   ,    ];
lastIndexExpected = [ 0, 1,    2,                        ];
ret = RegExp.prototype[Symbol.split].call(myRegExp, target);
assert.sameValue(arraySetterObserved, false);
assert.sameValue(JSON.stringify(ret), `["a","X","YZ","c"]`);
assert.sameValue(log,
         "get:constructor," +
         "get:species," +
         "get:flags," +
         "call:constructor," +
         "set:lastIndex,get:exec,call:exec," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "get:result[length]," +
         "get:result[1],get:result[2]," +
         "set:lastIndex,get:exec,call:exec,");

// Trace unicode.
// 1. not surrogate pair
// 2. lead surrogate pair
// 3. trail surrogate pair
// 4. lead surrogate pair without trail surrogate pair
// 5. index overflow
reset();
flags = "u";
expectedFlags = "uy";
target = "-\uD83D\uDC38\uDC38\uD83D";
execResult        = [    null, null, null, null ];
lastIndexResult   = [ ,  ,     ,     ,     ,    ];
lastIndexExpected = [ 0, 1,    3,    4,         ];
ret = RegExp.prototype[Symbol.split].call(myRegExp, target);
assert.sameValue(arraySetterObserved, false);
assert.sameValue(JSON.stringify(ret), `["-\uD83D\uDC38\\udc38\\ud83d"]`);
assert.sameValue(log,
         "get:constructor," +
         "get:species," +
         "get:flags," +
         "call:constructor," +
         "set:lastIndex,get:exec,call:exec," +
         "set:lastIndex,get:exec,call:exec," +
         "set:lastIndex,get:exec,call:exec," +
         "set:lastIndex,get:exec,call:exec,");

// Trace unicode, match, same position and different position.
reset();
flags = "u";
expectedFlags = "uy";
target = "-\uD83D\uDC38\uDC38\uD83D";
var E = P(["", "X"]);
execResult        = [    E, E, E, E, E, E, E ];
lastIndexResult   = [ ,  0, 1, 1, 3, 3, 4, 4 ];
lastIndexExpected = [ 0, 1, 1, 3, 3, 4, 4,   ];
ret = RegExp.prototype[Symbol.split].call(myRegExp, target);
assert.sameValue(arraySetterObserved, false);
assert.sameValue(JSON.stringify(ret), `["-","X","\uD83D\uDC38","X","\\udc38","X","\\ud83d"]`);
assert.sameValue(log,
         "get:constructor," +
         "get:species," +
         "get:flags," +
         "call:constructor," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "get:result[length]," +
         "get:result[1]," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "get:result[length]," +
         "get:result[1]," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex," +
         "get:result[length]," +
         "get:result[1]," +
         "set:lastIndex,get:exec,call:exec,get:lastIndex,");

stopObserve();
