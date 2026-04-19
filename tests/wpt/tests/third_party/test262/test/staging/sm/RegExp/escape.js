// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Slash and LineTerminator should be escaped correctly.
info: bugzilla.mozilla.org/show_bug.cgi?id=1130860
esid: pending
---*/

function test(re, source) {
  assert.sameValue(re.source, source);
  assert.sameValue(eval("/" + re.source + "/").source, source);
  assert.sameValue(re.toString(), "/" + source + "/");
}

test(/\\n/,           "\\\\n");
test(/\\\n/,          "\\\\\\n");
test(/\\\\n/,         "\\\\\\\\n");
test(RegExp("\\n"),   "\\n");
test(RegExp("\\\n"),  "\\n");
test(RegExp("\\\\n"), "\\\\n");

test(/\\r/,           "\\\\r");
test(/\\\r/,          "\\\\\\r");
test(/\\\\r/,         "\\\\\\\\r");
test(RegExp("\\r"),   "\\r");
test(RegExp("\\\r"),  "\\r");
test(RegExp("\\\\r"), "\\\\r");

test(/\\u2028/,           "\\\\u2028");
test(/\\\u2028/,          "\\\\\\u2028");
test(/\\\\u2028/,         "\\\\\\\\u2028");
test(RegExp("\\u2028"),   "\\u2028");
test(RegExp("\\\u2028"),  "\\u2028");
test(RegExp("\\\\u2028"), "\\\\u2028");

test(/\\u2029/,           "\\\\u2029");
test(/\\\u2029/,          "\\\\\\u2029");
test(/\\\\u2029/,         "\\\\\\\\u2029");
test(RegExp("\\u2029"),   "\\u2029");
test(RegExp("\\\u2029"),  "\\u2029");
test(RegExp("\\\\u2029"), "\\\\u2029");

test(/\//,            "\\/");
test(/\\\//,          "\\\\\\/");
test(RegExp("/"),     "\\/");
test(RegExp("\/"),    "\\/");
test(RegExp("\\/"),   "\\/");
test(RegExp("\\\/"),  "\\/");
test(RegExp("\\\\/"), "\\\\\\/");

test(/[/]/,             "[/]");
test(/[\/]/,            "[\\/]");
test(/[\\/]/,           "[\\\\/]");
test(/[\\\/]/,          "[\\\\\\/]");
test(RegExp("[/]"),     "[/]");
test(RegExp("[\/]"),    "[/]");
test(RegExp("[\\/]"),   "[\\/]");
test(RegExp("[\\\/]"),  "[\\/]");
test(RegExp("[\\\\/]"), "[\\\\/]");

test(RegExp("\[/\]"),   "[/]");
test(RegExp("\[\\/\]"), "[\\/]");

test(/\[\/\]/,              "\\[\\/\\]");
test(/\[\\\/\]/,            "\\[\\\\\\/\\]");
test(RegExp("\\[/\\]"),     "\\[\\/\\]");
test(RegExp("\\[\/\\]"),    "\\[\\/\\]");
test(RegExp("\\[\\/\\]"),   "\\[\\/\\]");
test(RegExp("\\[\\\/\\]"),  "\\[\\/\\]");
test(RegExp("\\[\\\\/\\]"), "\\[\\\\\\/\\]");
