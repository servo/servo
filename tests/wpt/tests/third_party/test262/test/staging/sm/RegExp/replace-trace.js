// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Trace RegExp.prototype[@@replace] behavior.
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

var n;
var log;
var target;
var global;
var unicode;

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

function P(A, index, matched2) {
  var i = 0;
  A.index = index;
  return new Proxy(A, {
    get(that, name) {
      log += "get:result[" + name + "],";

      // Return a different value for 2nd access to result[0].
      if (matched2 !== undefined && name == 0) {
        if (i == 1) {
          return matched2;
        }
        i++;
      }

      return that[name];
    }
  });
}

var myRegExp = {
  get flags() {
    log += "get:flags,";
    var flags = "";
    if (global) flags += "g";
    if (unicode) flags += "u";
    return flags;
  },
  get global() {
    // This getter should not be invoked, because we override the flags getter.
    log += "get:global,"
  },
  get unicode() {
    // This getter should not be invoked, because we override the flags getter.
    log += "get:unicode,"
  },
  get lastIndex() {
    log += "get:lastIndex,";
    return lastIndexResult[n];
  },
  set lastIndex(v) {
    log += "set:lastIndex,";
    assert.sameValue(v, lastIndexExpected[n]);
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

function reset() {
  n = 0;
  log = "";
  target = "abcAbcABC";
  global = true;
  unicode = false;
  arraySetterObserved = false;
}

// Trace global with non-empty match.
reset();
execResult        = [    P(["bc"], 1), null ];
lastIndexResult   = [ ,  ,                  ];
lastIndexExpected = [ 0, ,                  ];
var ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "_XYZ_");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "a_XYZ_AbcABC");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index],get:result[groups],");

// Trace global with empty match.
reset();
execResult        = [    P([""], 1), null ];
lastIndexResult   = [ ,  5,               ];
lastIndexExpected = [ 0, 6,               ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "_XYZ_");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "a_XYZ_bcAbcABC");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index],get:result[groups],");

// Trace global and unicode with empty match.
// 1. not surrogate pair
// 2. lead surrogate pair
// 3. trail surrogate pair
// 4. lead surrogate pair without trail surrogate pair
// 5. index overflow
reset();
unicode = true;
//        0123     4     5678
target = "---\uD83D\uDC38---\uD83D";
execResult        = [    P([""], 1), P([""], 2), P([""], 3), P([""], 4), P([""], 5), null ];
lastIndexResult   = [ ,  2,          3,          4,          8,          9,               ];
lastIndexExpected = [ 0, 3,          5,          5,          9,          10,              ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "_XYZ_");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "-_XYZ_-_XYZ_-_XYZ_\uD83D_XYZ_\uDC38_XYZ_---\uD83D");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index],get:result[groups]," +
         "get:result[length],get:result[0],get:result[index],get:result[groups]," +
         "get:result[length],get:result[0],get:result[index],get:result[groups]," +
         "get:result[length],get:result[0],get:result[index],get:result[groups]," +
         "get:result[length],get:result[0],get:result[index],get:result[groups],");

// Trace global with captures and substitutions.
reset();
execResult        = [    P(["bc", "b", "c"], 1), null ];
lastIndexResult   = [ ,  ,                            ];
lastIndexExpected = [ 0, ,                            ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "[$&,$`,$',$1,$2,$3,$]");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "a[bc,a,AbcABC,b,c,$3,$]AbcABC");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index]," +
         "get:result[1],get:result[2],get:result[groups],");

// Trace global with empty match and captures and substitutions,
// with different matched.
reset();
execResult        = [    P(["", "b", "c"], 1, "BC"), null ];
lastIndexResult   = [ ,  5,                               ];
lastIndexExpected = [ 0, 6,                               ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "[$&,$`,$',$1,$2,$3,$]");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "a[BC,a,AbcABC,b,c,$3,$]AbcABC");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index]," +
         "get:result[1],get:result[2],get:result[groups],");

// Trace global with empty match and captures and function,
// with different matched.
reset();
execResult        = [    P(["", "b", "c"], 1, "BC"), null ];
lastIndexResult   = [ ,  5,                               ];
lastIndexExpected = [ 0, 6,                               ];
function replaceFunc(...args) {
  log += "call:replaceFunc,";
  assert.sameValue(args.length, 5);
  assert.sameValue(args[0], "BC");
  assert.sameValue(args[1], "b");
  assert.sameValue(args[2], "c");
  assert.sameValue(args[3], 1);
  assert.sameValue(args[4], target);
  return "_ret_";
}
// This also tests RegExpStatics save/restore with no match.
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, replaceFunc);
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "a_ret_AbcABC");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:lastIndex,set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index]," +
         "get:result[1],get:result[2],get:result[groups]," +
         "call:replaceFunc,");

// Trace global with non-empty match, move backwards.
// 2nd match shouldn't be replaced.
reset();
execResult        = [    P(["X"], 5), P(["YZ"], 1), null ];
lastIndexResult   = [ ,  ,            ,                  ];
lastIndexExpected = [ 0, ,            ,                  ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "_XYZ_");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "abcAb_XYZ_ABC");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index],get:result[groups]," +
         "get:result[length],get:result[0],get:result[index],get:result[groups],");

// Trace global with non-empty match, position + matchLength overflows.
reset();
execResult        = [    P(["fooooooo"], 7), null ];
lastIndexResult   = [ ,  ,                        ];
lastIndexExpected = [ 0, ,                        ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "_XYZ_");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "abcAbcA_XYZ_");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index],get:result[groups],");

// Trace global with non-empty match, position overflows.
reset();
execResult        = [    P(["fooooooo"], 12), null ];
lastIndexResult   = [ ,  ,                         ];
lastIndexExpected = [ 0, ,                         ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "_XYZ_");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "abcAbcABC_XYZ_");
assert.sameValue(log,
         "get:flags," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:result[0]," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index],get:result[groups],");

// Trace non-global.
reset();
global = false;
execResult        = [    P(["bc"], 1) ];
lastIndexResult   = [ ,  ,            ];
lastIndexExpected = [ 0, ,            ];
ret = RegExp.prototype[Symbol.replace].call(myRegExp, target, "_XYZ_");
assert.sameValue(arraySetterObserved, false);
assert.sameValue(ret, "a_XYZ_AbcABC");
assert.sameValue(log,
         "get:flags," +
         "get:exec,call:exec," +
         "get:result[length],get:result[0],get:result[index],get:result[groups],");

stopObserve();
