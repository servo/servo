import {areArraysEqual} from '/common/arrays.js';

function parseNumber(value) {
  const num = parseInt(value.toString());
  if (isNaN(num)) return undefined;
  return num;
}

registerLayout('test', class {
  static get childInputProperties() {
    return [
      '--fixed-inline-size',
      '--fixed-block-size',
      '--inline-size-expected',
      '--block-size-expected'
    ];
  }

  *intrinsicSizes() {}
  *layout(children, edges, constraints, styleMap) {
    const childFragments = yield children.map((child) => {
      const childConstraints = {};
      const fixedInlineSize = parseNumber(child.styleMap.get('--fixed-inline-size'));
      const fixedBlockSize = parseNumber(child.styleMap.get('--fixed-block-size'));
      return child.layoutNextFragment({fixedInlineSize, fixedBlockSize});
    });

    const actual = childFragments.map((childFragment) => {
      return {
        inlineSize: childFragment.inlineSize,
        blockSize: childFragment.blockSize,
      };
    });

    const expected = children.map((child) => {
      return {
        inlineSize: parseInt(child.styleMap.get('--inline-size-expected').toString()),
        blockSize: parseInt(child.styleMap.get('--block-size-expected').toString()),
      };
    });

    const equalityFunc = (a, b) => {
      return a.inlineSize == b.inlineSize && a.blockSize == b.blockSize;
    };

    if (!areArraysEqual(expected, actual, equalityFunc)) {
      return {autoBlockSize: 0, childFragments};
    }

    return {autoBlockSize: 100, childFragments};
  }
});
