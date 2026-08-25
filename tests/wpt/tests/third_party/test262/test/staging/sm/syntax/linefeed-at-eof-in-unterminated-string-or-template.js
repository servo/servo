/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Properly handle the case of U+005C REVERSE SOLIDUS U+000D CARRIAGE RETURN at the end of source text being tokenized, in the middle of a string or template literal, where the next code point in memory (outside the bounds of the source text) is U+000A LINE FEED
info: bugzilla.mozilla.org/show_bug.cgi?id=1476409
esid: pending
---*/

function expectSyntaxError(code)
{
  assert.throws(SyntaxError, function() {
    eval(code);
  });
}

// The fundamental requirements of this test:
//
// 1. The computed string that is eval'd must be a Script that ends in a string
//    literal ending with the code points U+005C REVERSE SOLIDUS U+000D CARRIAGE
//    RETURN.
// 2. The *memory* that is actually tokenized/parsed by eval must be
//    immediately followed by U+000A LINE FEED.
//
// There's only one way to guarantee a U+000A LINE FEED after the source text:
// compute the source text as a dependent string,  of a larger (linear) string.
//  A simple substr will do the trick -- just as long as the substring can't fit
// in inline storage.  53 in the tests below comfortably exceeds all inline
// storage limits.
//
// One final wrinkle: because we only tokenize/parse two-byte source text right
// now, ensuring we directly tokenize/parse the dependent string's character
// data means the dependent string must have two-byte character data, hence the
// '\u1234' in the strings below.

function singleQuote()
{
  var containsBadSingleQuoteLiteral =
    "\u1234x'01234567890123456789012345678901234567890123456789\\\r\n0123456789";
  //        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

  expectSyntaxError(containsBadSingleQuoteLiteral.substr(2, 53));
}
singleQuote();

function doubleQuote()
{
  var containsBadDoubleQuoteLiteral =
    "\u1234x\"01234567890123456789012345678901234567890123456789\\\r\n0123456789";
  //        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

  expectSyntaxError(containsBadDoubleQuoteLiteral.substr(2, 53));
}
doubleQuote();

function template()
{
  var containsBadTemplateLiteral =
    "\u1234x`01234567890123456789012345678901234567890123456789\\\r\n0123456789";
  //        ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

  expectSyntaxError(containsBadTemplateLiteral.substr(2, 53));
}
template();
