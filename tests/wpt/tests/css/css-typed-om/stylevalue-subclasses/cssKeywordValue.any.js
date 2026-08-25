// META: global=window,worker
// META: title=CSSKeywordValue Constructor
// META: spec=https://drafts.css-houdini.org/css-typed-om-1/#dom-csskeywordvalue-value

'use strict';

const gTestArguments = [
  { keyword: 'initial', description: 'a CSS wide keyword' },
  { keyword: 'auto', description: 'a CSS keyword' },
  { keyword: 'lemon', description: 'an unsupported CSS keyword' },
  { keyword: '3! + 4@', description: 'a string containing multiple tokens' },
  { keyword: '☺', description: 'a unicode string' },
];

for (const args of gTestArguments) {
  test(() => {
    const result = new CSSKeywordValue(args.keyword);
    assert_not_equals(result, null, 'a CSSKeywordValue is created');
    assert_equals(result.value, args.keyword,
                  'value is same as given by constructor');
  }, `CSSKeywordValue can be constructed from ${args.description}`);
}
