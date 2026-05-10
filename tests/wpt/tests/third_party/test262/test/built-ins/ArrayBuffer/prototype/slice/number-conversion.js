// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-arraybuffer.prototype.slice
description: >
  ToInteger(start) is called before ToInteger(end).
info: |
  ArrayBuffer.prototype.slice ( start, end )

  ...
  6. Let relativeStart be ToInteger(start).
  7. ReturnIfAbrupt(relativeStart).
  ...
  9. If end is undefined, let relativeEnd be len; else let relativeEnd be ToInteger(end).
  10. ReturnIfAbrupt(relativeEnd).
  ...
---*/

var arrayBuffer = new ArrayBuffer(8);

var log = "";
var start = {
  valueOf: function() {
    log += "start-";
    return 0;
  }
};
var end = {
  valueOf: function() {
    log += "end";
    return 8;
  }
};

arrayBuffer.slice(start, end);
assert.sameValue(log, "start-end");
