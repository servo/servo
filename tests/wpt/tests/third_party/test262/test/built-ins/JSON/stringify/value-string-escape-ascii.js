// Copyright (c) 2018 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-quotejsonstring
description: >
    JSON.stringify property names and values containing ASCII
    characters that require escaping
---*/

var char_to_json = {
  '"': '\\"',
  "\\": "\\\\",
  "\x00": "\\u0000",
  "\x01": "\\u0001",
  "\x02": "\\u0002",
  "\x03": "\\u0003",
  "\x04": "\\u0004",
  "\x05": "\\u0005",
  "\x06": "\\u0006",
  "\x07": "\\u0007",
  "\x08": "\\b",
  "\x09": "\\t",
  "\x0A": "\\n",
  "\x0B": "\\u000b",
  "\x0C": "\\f",
  "\x0D": "\\r",
  "\x0E": "\\u000e",
  "\x0F": "\\u000f",
  "\x10": "\\u0010",
  "\x11": "\\u0011",
  "\x12": "\\u0012",
  "\x13": "\\u0013",
  "\x14": "\\u0014",
  "\x15": "\\u0015",
  "\x16": "\\u0016",
  "\x17": "\\u0017",
  "\x18": "\\u0018",
  "\x19": "\\u0019",
  "\x1A": "\\u001a",
  "\x1B": "\\u001b",
  "\x1C": "\\u001c",
  "\x1D": "\\u001d",
  "\x1E": "\\u001e",
  "\x1F": "\\u001f"
}

var chars = Object.keys(char_to_json).join("");
var chars_reversed = Object.keys(char_to_json).reverse().join("");
var jsonChars = Object.values(char_to_json).join("");
var jsonChars_reversed = Object.values(char_to_json).reverse().join("");
var json = JSON.stringify({
  ["name" + chars + chars_reversed]: chars_reversed + chars + "value"
});

for (var char in char_to_json) {
  var count = json.split(char_to_json[char]).length - 1;
  assert.sameValue(count, 4,
    "Every ASCII 0x" + char.charCodeAt(0).toString(16) + " serializes to " + char_to_json[char]);
}

assert.sameValue(
  json,
  `{"${"name" + jsonChars + jsonChars_reversed}":"${jsonChars_reversed + jsonChars + "value"}"}`,
  "JSON.stringify(objectUsingControlCharacters)"
);
