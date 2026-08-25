// Copyright (c) 2017 Valerie Young.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-string.prototype.trimstart
description: Behavior when "this" value is a boolean.
info: |
  Runtime Semantics: TrimString ( string, where )
  2. Let S be ? ToString(str).

  ToString ( argument )
  Argument Type: Boolean
  Result:
    If argument is true, return "true".
    If argument is false, return "false".
features: [string-trimming, String.prototype.trimStart]
---*/

var trimStart = String.prototype.trimStart

assert.sameValue(
  trimStart.call(true),
  'true',
  'String.prototype.trimStart.call(true)'
);

assert.sameValue(
  String.prototype.trimStart.call(false),
  'false',
  'String.prototype.trimStart.call(false)'
);
