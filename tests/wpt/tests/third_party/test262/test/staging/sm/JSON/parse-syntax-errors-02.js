// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-JSON-shell.js]
description: |
  pending
esid: pending
---*/

testJSONSyntaxError('"Unterminated string literal');
testJSONSyntaxError('["Unclosed array"');
testJSONSyntaxError('{unquoted_key: "keys must be quoted"}');
testJSONSyntaxError('["extra comma",]');
testJSONSyntaxError('["double extra comma",,]');
testJSONSyntaxError('[   , "<-- missing value"]');
testJSONSyntaxError('["Comma after the close"],');
testJSONSyntaxError('["Extra close"]]');
testJSONSyntaxError('{"Extra comma": true,}');
testJSONSyntaxError('{"Extra value after close": true} "misplaced quoted value"');
testJSONSyntaxError('{"Illegal expression": 1 + 2}');
testJSONSyntaxError('{"Illegal invocation": alert()}');
testJSONSyntaxError('{"Numbers cannot be hex": 0x14}');
testJSONSyntaxError('["Illegal backslash escape: \\x15"]');
testJSONSyntaxError('[\\naked]');
testJSONSyntaxError('["Illegal backslash escape: \\017"]');
testJSONSyntaxError('{"Missing colon" null}');
testJSONSyntaxError('{"Double colon":: null}');
testJSONSyntaxError('{"Comma instead of colon", null}');
testJSONSyntaxError('["Colon instead of comma": false]');
testJSONSyntaxError('["Bad value", truth]');
testJSONSyntaxError("['single quote']");
testJSONSyntaxError('["	tab	character	in	string	"]');
testJSONSyntaxError('["tab\\   character\\   in\\  string\\  "]');
testJSONSyntaxError('["line\rbreak"]');
testJSONSyntaxError('["line\nbreak"]');
testJSONSyntaxError('["line\r\nbreak"]');
testJSONSyntaxError('["line\\\rbreak"]');
testJSONSyntaxError('["line\\\nbreak"]');
testJSONSyntaxError('["line\\\r\nbreak"]');
testJSONSyntaxError('[0e]');
testJSONSyntaxError('[0e+]');
testJSONSyntaxError('[0e+-1]');
testJSONSyntaxError('{"Comma instead of closing brace": true,');
testJSONSyntaxError('["mismatch"}');
testJSONSyntaxError('0{');
