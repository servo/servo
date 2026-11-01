// META: global=window,worker
// META: title=CSSUnparsedValue: Don't crash for empty values
// META: spec=https://drafts.css-houdini.org/css-typed-om-1/#dom-cssunparsedvalue-cssunparsedvalue

'use strict';

// https://crbug.com/1169941
test(() => {
  const result = new CSSUnparsedValue(['']);
  assert_equals('', result.toString()); // Don't crash.
}, `Don't crash when serializing empty CSSUnparsedValue`);

test(() => {
  const validSelector =
      new CSSUnparsedValue(['if(supports(selector(div)): 10px)']);
  assert_equals('if(supports(selector(div)): 10px)', validSelector.toString());

  const complexSelector =
      new CSSUnparsedValue(['if(supports(selector(x >)): 1px)']);
  assert_equals('if(supports(selector(x >)): 1px)', complexSelector.toString());

  const supportsProperty =
      new CSSUnparsedValue(['if(supports(list-style-type: disc): 10px)']);
  assert_equals('if(supports(list-style-type: disc): 10px)',
                supportsProperty.toString());

  const incompleteVar = new CSSUnparsedValue(['var()']);
  assert_equals('var()', incompleteVar.toString());
}, `Serializing CSSUnparsedValue with conditional and substitution syntax`);
