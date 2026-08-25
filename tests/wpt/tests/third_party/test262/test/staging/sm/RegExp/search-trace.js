// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Trace RegExp.prototype[@@search] behavior.
info: bugzilla.mozilla.org/show_bug.cgi?id=887016
esid: pending
---*/

var n;
var log;
var target;

var execResult;
var lastIndexResult;
var lastIndexExpected;

function P(index) {
  return new Proxy({ index }, {
    get(that, name) {
      log += "get:result[" + name + "],";
      return that[name];
    }
  });
}

var myRegExp = {
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
}

// Trace hit.
reset();
execResult        = [     P(16) ];
lastIndexResult   = [ 10, ,     ];
lastIndexExpected = [ 0,  10    ];
var ret = RegExp.prototype[Symbol.search].call(myRegExp, target);
assert.sameValue(ret, 16);
assert.sameValue(log,
         "get:lastIndex," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:lastIndex," +
         "set:lastIndex," +
         "get:result[index],");

// Trace not hit.
reset();
execResult        = [     null ];
lastIndexResult   = [ 10, ,    ];
lastIndexExpected = [ 0,  10   ];
ret = RegExp.prototype[Symbol.search].call(myRegExp, target);
assert.sameValue(ret, -1);
assert.sameValue(log,
         "get:lastIndex," +
         "set:lastIndex," +
         "get:exec,call:exec," +
         "get:lastIndex," +
         "set:lastIndex,");
