// Copyright (C) 2017 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: prod-Template
description: NumericLiteralSeperator disallowed in unicode CodePoint sequence
info: |
 Template ::
   NoSubstitutionTemplate
   TemplateHead

 NoSubstitutionTemplate ::
   `TemplateCharactersopt`

 TemplateCharacters ::
   TemplateCharacterTemplateCharactersopt

 TemplateCharacter ::
   $[lookahead ≠ {]
   \EscapeSequence
   \NotEscapeSequence
   LineContinuation
   LineTerminatorSequence
   SourceCharacterbut not one of ` or \ or $ or LineTerminator

 EscapeSequence ::
   CharacterEscapeSequence
   0 [lookahead ∉ DecimalDigit]
   HexEscapeSequence
   UnicodeEscapeSequence

 UnicodeEscapeSequence ::
   uHex4Digits
   u{CodePoint}

 CodePoint ::
   HexDigit but only if MV of HexDigits ≤ 0x10FFFF
   CodePointDigits but only if MV of HexDigits ≤ 0x10FFFF

 CodePointDigits ::
   HexDigit
   CodePointDigitsHexDigit

  HexDigit :: one of
    0 1 2 3 4 5 6 7 8 9 a b c d e f A B C D E F

features: [numeric-separator-literal]
negative:
  phase: parse
  type: SyntaxError
---*/

$DONOTEVALUATE();

`\u{1F_639}`;
