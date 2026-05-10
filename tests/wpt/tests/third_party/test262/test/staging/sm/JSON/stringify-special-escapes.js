/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  JSON.stringify of \\b\\f\\n\\r\\t should use one-character escapes, not hex
info: bugzilla.mozilla.org/show_bug.cgi?id=512266
esid: pending
---*/

assert.sameValue(JSON.stringify("\u0000"), '"\\u0000"');
assert.sameValue(JSON.stringify("\u0001"), '"\\u0001"');
assert.sameValue(JSON.stringify("\u0002"), '"\\u0002"');
assert.sameValue(JSON.stringify("\u0003"), '"\\u0003"');
assert.sameValue(JSON.stringify("\u0004"), '"\\u0004"');
assert.sameValue(JSON.stringify("\u0005"), '"\\u0005"');
assert.sameValue(JSON.stringify("\u0006"), '"\\u0006"');
assert.sameValue(JSON.stringify("\u0007"), '"\\u0007"');
assert.sameValue(JSON.stringify("\u0008"), '"\\b"');
assert.sameValue(JSON.stringify("\u0009"), '"\\t"');
assert.sameValue(JSON.stringify("\u000A"), '"\\n"');
assert.sameValue(JSON.stringify("\u000B"), '"\\u000b"');
assert.sameValue(JSON.stringify("\u000C"), '"\\f"');
assert.sameValue(JSON.stringify("\u000D"), '"\\r"');
assert.sameValue(JSON.stringify("\u000E"), '"\\u000e"');
assert.sameValue(JSON.stringify("\u000F"), '"\\u000f"');
assert.sameValue(JSON.stringify("\u0010"), '"\\u0010"');
assert.sameValue(JSON.stringify("\u0011"), '"\\u0011"');
assert.sameValue(JSON.stringify("\u0012"), '"\\u0012"');
assert.sameValue(JSON.stringify("\u0013"), '"\\u0013"');
assert.sameValue(JSON.stringify("\u0014"), '"\\u0014"');
assert.sameValue(JSON.stringify("\u0015"), '"\\u0015"');
assert.sameValue(JSON.stringify("\u0016"), '"\\u0016"');
assert.sameValue(JSON.stringify("\u0017"), '"\\u0017"');
assert.sameValue(JSON.stringify("\u0018"), '"\\u0018"');
assert.sameValue(JSON.stringify("\u0019"), '"\\u0019"');
assert.sameValue(JSON.stringify("\u001A"), '"\\u001a"');
assert.sameValue(JSON.stringify("\u001B"), '"\\u001b"');
assert.sameValue(JSON.stringify("\u001C"), '"\\u001c"');
assert.sameValue(JSON.stringify("\u001D"), '"\\u001d"');
assert.sameValue(JSON.stringify("\u001E"), '"\\u001e"');
assert.sameValue(JSON.stringify("\u001F"), '"\\u001f"');
assert.sameValue(JSON.stringify("\u0020"), '" "');

assert.sameValue(JSON.stringify("\\u0000"), '"\\\\u0000"');
assert.sameValue(JSON.stringify("\\u0001"), '"\\\\u0001"');
assert.sameValue(JSON.stringify("\\u0002"), '"\\\\u0002"');
assert.sameValue(JSON.stringify("\\u0003"), '"\\\\u0003"');
assert.sameValue(JSON.stringify("\\u0004"), '"\\\\u0004"');
assert.sameValue(JSON.stringify("\\u0005"), '"\\\\u0005"');
assert.sameValue(JSON.stringify("\\u0006"), '"\\\\u0006"');
assert.sameValue(JSON.stringify("\\u0007"), '"\\\\u0007"');
assert.sameValue(JSON.stringify("\\u0008"), '"\\\\u0008"');
assert.sameValue(JSON.stringify("\\u0009"), '"\\\\u0009"');
assert.sameValue(JSON.stringify("\\u000A"), '"\\\\u000A"');
assert.sameValue(JSON.stringify("\\u000B"), '"\\\\u000B"');
assert.sameValue(JSON.stringify("\\u000C"), '"\\\\u000C"');
assert.sameValue(JSON.stringify("\\u000D"), '"\\\\u000D"');
assert.sameValue(JSON.stringify("\\u000E"), '"\\\\u000E"');
assert.sameValue(JSON.stringify("\\u000F"), '"\\\\u000F"');
assert.sameValue(JSON.stringify("\\u0010"), '"\\\\u0010"');
assert.sameValue(JSON.stringify("\\u0011"), '"\\\\u0011"');
assert.sameValue(JSON.stringify("\\u0012"), '"\\\\u0012"');
assert.sameValue(JSON.stringify("\\u0013"), '"\\\\u0013"');
assert.sameValue(JSON.stringify("\\u0014"), '"\\\\u0014"');
assert.sameValue(JSON.stringify("\\u0015"), '"\\\\u0015"');
assert.sameValue(JSON.stringify("\\u0016"), '"\\\\u0016"');
assert.sameValue(JSON.stringify("\\u0017"), '"\\\\u0017"');
assert.sameValue(JSON.stringify("\\u0018"), '"\\\\u0018"');
assert.sameValue(JSON.stringify("\\u0019"), '"\\\\u0019"');
assert.sameValue(JSON.stringify("\\u001A"), '"\\\\u001A"');
assert.sameValue(JSON.stringify("\\u001B"), '"\\\\u001B"');
assert.sameValue(JSON.stringify("\\u001C"), '"\\\\u001C"');
assert.sameValue(JSON.stringify("\\u001D"), '"\\\\u001D"');
assert.sameValue(JSON.stringify("\\u001E"), '"\\\\u001E"');
assert.sameValue(JSON.stringify("\\u001F"), '"\\\\u001F"');
assert.sameValue(JSON.stringify("\\u0020"), '"\\\\u0020"');


assert.sameValue(JSON.stringify("a\u0000"), '"a\\u0000"');
assert.sameValue(JSON.stringify("a\u0001"), '"a\\u0001"');
assert.sameValue(JSON.stringify("a\u0002"), '"a\\u0002"');
assert.sameValue(JSON.stringify("a\u0003"), '"a\\u0003"');
assert.sameValue(JSON.stringify("a\u0004"), '"a\\u0004"');
assert.sameValue(JSON.stringify("a\u0005"), '"a\\u0005"');
assert.sameValue(JSON.stringify("a\u0006"), '"a\\u0006"');
assert.sameValue(JSON.stringify("a\u0007"), '"a\\u0007"');
assert.sameValue(JSON.stringify("a\u0008"), '"a\\b"');
assert.sameValue(JSON.stringify("a\u0009"), '"a\\t"');
assert.sameValue(JSON.stringify("a\u000A"), '"a\\n"');
assert.sameValue(JSON.stringify("a\u000B"), '"a\\u000b"');
assert.sameValue(JSON.stringify("a\u000C"), '"a\\f"');
assert.sameValue(JSON.stringify("a\u000D"), '"a\\r"');
assert.sameValue(JSON.stringify("a\u000E"), '"a\\u000e"');
assert.sameValue(JSON.stringify("a\u000F"), '"a\\u000f"');
assert.sameValue(JSON.stringify("a\u0010"), '"a\\u0010"');
assert.sameValue(JSON.stringify("a\u0011"), '"a\\u0011"');
assert.sameValue(JSON.stringify("a\u0012"), '"a\\u0012"');
assert.sameValue(JSON.stringify("a\u0013"), '"a\\u0013"');
assert.sameValue(JSON.stringify("a\u0014"), '"a\\u0014"');
assert.sameValue(JSON.stringify("a\u0015"), '"a\\u0015"');
assert.sameValue(JSON.stringify("a\u0016"), '"a\\u0016"');
assert.sameValue(JSON.stringify("a\u0017"), '"a\\u0017"');
assert.sameValue(JSON.stringify("a\u0018"), '"a\\u0018"');
assert.sameValue(JSON.stringify("a\u0019"), '"a\\u0019"');
assert.sameValue(JSON.stringify("a\u001A"), '"a\\u001a"');
assert.sameValue(JSON.stringify("a\u001B"), '"a\\u001b"');
assert.sameValue(JSON.stringify("a\u001C"), '"a\\u001c"');
assert.sameValue(JSON.stringify("a\u001D"), '"a\\u001d"');
assert.sameValue(JSON.stringify("a\u001E"), '"a\\u001e"');
assert.sameValue(JSON.stringify("a\u001F"), '"a\\u001f"');
assert.sameValue(JSON.stringify("a\u0020"), '"a "');

assert.sameValue(JSON.stringify("a\\u0000"), '"a\\\\u0000"');
assert.sameValue(JSON.stringify("a\\u0001"), '"a\\\\u0001"');
assert.sameValue(JSON.stringify("a\\u0002"), '"a\\\\u0002"');
assert.sameValue(JSON.stringify("a\\u0003"), '"a\\\\u0003"');
assert.sameValue(JSON.stringify("a\\u0004"), '"a\\\\u0004"');
assert.sameValue(JSON.stringify("a\\u0005"), '"a\\\\u0005"');
assert.sameValue(JSON.stringify("a\\u0006"), '"a\\\\u0006"');
assert.sameValue(JSON.stringify("a\\u0007"), '"a\\\\u0007"');
assert.sameValue(JSON.stringify("a\\u0008"), '"a\\\\u0008"');
assert.sameValue(JSON.stringify("a\\u0009"), '"a\\\\u0009"');
assert.sameValue(JSON.stringify("a\\u000A"), '"a\\\\u000A"');
assert.sameValue(JSON.stringify("a\\u000B"), '"a\\\\u000B"');
assert.sameValue(JSON.stringify("a\\u000C"), '"a\\\\u000C"');
assert.sameValue(JSON.stringify("a\\u000D"), '"a\\\\u000D"');
assert.sameValue(JSON.stringify("a\\u000E"), '"a\\\\u000E"');
assert.sameValue(JSON.stringify("a\\u000F"), '"a\\\\u000F"');
assert.sameValue(JSON.stringify("a\\u0010"), '"a\\\\u0010"');
assert.sameValue(JSON.stringify("a\\u0011"), '"a\\\\u0011"');
assert.sameValue(JSON.stringify("a\\u0012"), '"a\\\\u0012"');
assert.sameValue(JSON.stringify("a\\u0013"), '"a\\\\u0013"');
assert.sameValue(JSON.stringify("a\\u0014"), '"a\\\\u0014"');
assert.sameValue(JSON.stringify("a\\u0015"), '"a\\\\u0015"');
assert.sameValue(JSON.stringify("a\\u0016"), '"a\\\\u0016"');
assert.sameValue(JSON.stringify("a\\u0017"), '"a\\\\u0017"');
assert.sameValue(JSON.stringify("a\\u0018"), '"a\\\\u0018"');
assert.sameValue(JSON.stringify("a\\u0019"), '"a\\\\u0019"');
assert.sameValue(JSON.stringify("a\\u001A"), '"a\\\\u001A"');
assert.sameValue(JSON.stringify("a\\u001B"), '"a\\\\u001B"');
assert.sameValue(JSON.stringify("a\\u001C"), '"a\\\\u001C"');
assert.sameValue(JSON.stringify("a\\u001D"), '"a\\\\u001D"');
assert.sameValue(JSON.stringify("a\\u001E"), '"a\\\\u001E"');
assert.sameValue(JSON.stringify("a\\u001F"), '"a\\\\u001F"');
assert.sameValue(JSON.stringify("a\\u0020"), '"a\\\\u0020"');


assert.sameValue(JSON.stringify("\u0000Q"), '"\\u0000Q"');
assert.sameValue(JSON.stringify("\u0001Q"), '"\\u0001Q"');
assert.sameValue(JSON.stringify("\u0002Q"), '"\\u0002Q"');
assert.sameValue(JSON.stringify("\u0003Q"), '"\\u0003Q"');
assert.sameValue(JSON.stringify("\u0004Q"), '"\\u0004Q"');
assert.sameValue(JSON.stringify("\u0005Q"), '"\\u0005Q"');
assert.sameValue(JSON.stringify("\u0006Q"), '"\\u0006Q"');
assert.sameValue(JSON.stringify("\u0007Q"), '"\\u0007Q"');
assert.sameValue(JSON.stringify("\u0008Q"), '"\\bQ"');
assert.sameValue(JSON.stringify("\u0009Q"), '"\\tQ"');
assert.sameValue(JSON.stringify("\u000AQ"), '"\\nQ"');
assert.sameValue(JSON.stringify("\u000BQ"), '"\\u000bQ"');
assert.sameValue(JSON.stringify("\u000CQ"), '"\\fQ"');
assert.sameValue(JSON.stringify("\u000DQ"), '"\\rQ"');
assert.sameValue(JSON.stringify("\u000EQ"), '"\\u000eQ"');
assert.sameValue(JSON.stringify("\u000FQ"), '"\\u000fQ"');
assert.sameValue(JSON.stringify("\u0010Q"), '"\\u0010Q"');
assert.sameValue(JSON.stringify("\u0011Q"), '"\\u0011Q"');
assert.sameValue(JSON.stringify("\u0012Q"), '"\\u0012Q"');
assert.sameValue(JSON.stringify("\u0013Q"), '"\\u0013Q"');
assert.sameValue(JSON.stringify("\u0014Q"), '"\\u0014Q"');
assert.sameValue(JSON.stringify("\u0015Q"), '"\\u0015Q"');
assert.sameValue(JSON.stringify("\u0016Q"), '"\\u0016Q"');
assert.sameValue(JSON.stringify("\u0017Q"), '"\\u0017Q"');
assert.sameValue(JSON.stringify("\u0018Q"), '"\\u0018Q"');
assert.sameValue(JSON.stringify("\u0019Q"), '"\\u0019Q"');
assert.sameValue(JSON.stringify("\u001AQ"), '"\\u001aQ"');
assert.sameValue(JSON.stringify("\u001BQ"), '"\\u001bQ"');
assert.sameValue(JSON.stringify("\u001CQ"), '"\\u001cQ"');
assert.sameValue(JSON.stringify("\u001DQ"), '"\\u001dQ"');
assert.sameValue(JSON.stringify("\u001EQ"), '"\\u001eQ"');
assert.sameValue(JSON.stringify("\u001FQ"), '"\\u001fQ"');
assert.sameValue(JSON.stringify("\u0020Q"), '" Q"');

assert.sameValue(JSON.stringify("\\u0000Q"), '"\\\\u0000Q"');
assert.sameValue(JSON.stringify("\\u0001Q"), '"\\\\u0001Q"');
assert.sameValue(JSON.stringify("\\u0002Q"), '"\\\\u0002Q"');
assert.sameValue(JSON.stringify("\\u0003Q"), '"\\\\u0003Q"');
assert.sameValue(JSON.stringify("\\u0004Q"), '"\\\\u0004Q"');
assert.sameValue(JSON.stringify("\\u0005Q"), '"\\\\u0005Q"');
assert.sameValue(JSON.stringify("\\u0006Q"), '"\\\\u0006Q"');
assert.sameValue(JSON.stringify("\\u0007Q"), '"\\\\u0007Q"');
assert.sameValue(JSON.stringify("\\u0008Q"), '"\\\\u0008Q"');
assert.sameValue(JSON.stringify("\\u0009Q"), '"\\\\u0009Q"');
assert.sameValue(JSON.stringify("\\u000AQ"), '"\\\\u000AQ"');
assert.sameValue(JSON.stringify("\\u000BQ"), '"\\\\u000BQ"');
assert.sameValue(JSON.stringify("\\u000CQ"), '"\\\\u000CQ"');
assert.sameValue(JSON.stringify("\\u000DQ"), '"\\\\u000DQ"');
assert.sameValue(JSON.stringify("\\u000EQ"), '"\\\\u000EQ"');
assert.sameValue(JSON.stringify("\\u000FQ"), '"\\\\u000FQ"');
assert.sameValue(JSON.stringify("\\u0010Q"), '"\\\\u0010Q"');
assert.sameValue(JSON.stringify("\\u0011Q"), '"\\\\u0011Q"');
assert.sameValue(JSON.stringify("\\u0012Q"), '"\\\\u0012Q"');
assert.sameValue(JSON.stringify("\\u0013Q"), '"\\\\u0013Q"');
assert.sameValue(JSON.stringify("\\u0014Q"), '"\\\\u0014Q"');
assert.sameValue(JSON.stringify("\\u0015Q"), '"\\\\u0015Q"');
assert.sameValue(JSON.stringify("\\u0016Q"), '"\\\\u0016Q"');
assert.sameValue(JSON.stringify("\\u0017Q"), '"\\\\u0017Q"');
assert.sameValue(JSON.stringify("\\u0018Q"), '"\\\\u0018Q"');
assert.sameValue(JSON.stringify("\\u0019Q"), '"\\\\u0019Q"');
assert.sameValue(JSON.stringify("\\u001AQ"), '"\\\\u001AQ"');
assert.sameValue(JSON.stringify("\\u001BQ"), '"\\\\u001BQ"');
assert.sameValue(JSON.stringify("\\u001CQ"), '"\\\\u001CQ"');
assert.sameValue(JSON.stringify("\\u001DQ"), '"\\\\u001DQ"');
assert.sameValue(JSON.stringify("\\u001EQ"), '"\\\\u001EQ"');
assert.sameValue(JSON.stringify("\\u001FQ"), '"\\\\u001FQ"');
assert.sameValue(JSON.stringify("\\u0020Q"), '"\\\\u0020Q"');

// https://tc39.github.io/proposal-well-formed-stringify/

assert.sameValue(JSON.stringify("\ud7ff"), '"\ud7ff"');
assert.sameValue(JSON.stringify("\ud800"), '"\\ud800"');
assert.sameValue(JSON.stringify("\ud937"), '"\\ud937"');
assert.sameValue(JSON.stringify("\uda20"), '"\\uda20"');
assert.sameValue(JSON.stringify("\udbff"), '"\\udbff"');

assert.sameValue(JSON.stringify("\udc00"), '"\\udc00"');
assert.sameValue(JSON.stringify("\udddd"), '"\\udddd"');
assert.sameValue(JSON.stringify("\udeaf"), '"\\udeaf"');
assert.sameValue(JSON.stringify("\udfff"), '"\\udfff"');
assert.sameValue(JSON.stringify("\ue000"), '"\ue000"');

assert.sameValue(JSON.stringify("\ud7ffa"), '"\ud7ffa"');
assert.sameValue(JSON.stringify("\ud800a"), '"\\ud800a"');
assert.sameValue(JSON.stringify("\ud937a"), '"\\ud937a"');
assert.sameValue(JSON.stringify("\uda20a"), '"\\uda20a"');
assert.sameValue(JSON.stringify("\udbffa"), '"\\udbffa"');

assert.sameValue(JSON.stringify("\udc00a"), '"\\udc00a"');
assert.sameValue(JSON.stringify("\udddda"), '"\\udddda"');
assert.sameValue(JSON.stringify("\udeafa"), '"\\udeafa"');
assert.sameValue(JSON.stringify("\udfffa"), '"\\udfffa"');
assert.sameValue(JSON.stringify("\ue000a"), '"\ue000a"');

assert.sameValue(JSON.stringify("\ud7ff\ud800"), '"\ud7ff\\ud800"');
assert.sameValue(JSON.stringify("\ud800\ud800"), '"\\ud800\\ud800"');
assert.sameValue(JSON.stringify("\ud937\ud800"), '"\\ud937\\ud800"');
assert.sameValue(JSON.stringify("\uda20\ud800"), '"\\uda20\\ud800"');
assert.sameValue(JSON.stringify("\udbff\ud800"), '"\\udbff\\ud800"');

assert.sameValue(JSON.stringify("\udc00\ud800"), '"\\udc00\\ud800"');
assert.sameValue(JSON.stringify("\udddd\ud800"), '"\\udddd\\ud800"');
assert.sameValue(JSON.stringify("\udeaf\ud800"), '"\\udeaf\\ud800"');
assert.sameValue(JSON.stringify("\udfff\ud800"), '"\\udfff\\ud800"');
assert.sameValue(JSON.stringify("\ue000\ud800"), '"\ue000\\ud800"');

assert.sameValue(JSON.stringify("\ud7ff\udc00"), '"\ud7ff\\udc00"');
assert.sameValue(JSON.stringify("\ud800\udc00"), '"\ud800\udc00"');
assert.sameValue(JSON.stringify("\ud937\udc00"), '"\ud937\udc00"');
assert.sameValue(JSON.stringify("\uda20\udc00"), '"\uda20\udc00"');
assert.sameValue(JSON.stringify("\udbff\udc00"), '"\udbff\udc00"');

assert.sameValue(JSON.stringify("\udc00\udc00"), '"\\udc00\\udc00"');
assert.sameValue(JSON.stringify("\udddd\udc00"), '"\\udddd\\udc00"');
assert.sameValue(JSON.stringify("\udeaf\udc00"), '"\\udeaf\\udc00"');
assert.sameValue(JSON.stringify("\udfff\udc00"), '"\\udfff\\udc00"');
assert.sameValue(JSON.stringify("\ue000\udc00"), '"\ue000\\udc00"');
