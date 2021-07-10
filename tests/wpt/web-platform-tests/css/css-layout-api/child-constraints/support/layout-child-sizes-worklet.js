import {areArraysEqual} from '/common/arrays.js';

function parseNumber(value) {
  const num = parseInt(value.toString());
  if (isNaN(num)) return undefined;
  return num;
}

registerLayout('test', class {
  static get childInputProperties() {
    return [
      '--available-inline-size',
      '--available-block-size',
      '--fixed-inline-size',
      '--fixed-block-size',
      '--percentage-inline-size',
      '--percentage-block-size',
      '--inline-size-expected',
      '--block-size-expected'
    ];
  }

  async intrinsicSizes() {}
  async layout(children, edges, constraints, styleMap) {
    const childFragments = await Promise.all(children.map((child) => {
      const childConstraints = {};
      const availableInlineSize = parseNumber(child.styleMap.get('--available-inline-size'));
      const availableBlockSize = parseNumber(child.styleMap.get('--available-block-size'));
      const fixedInlineSize = parseNumber(child.styleMap.get('--fixed-inline-size'));
      const fixedBlockSize = parseNumber(child.styleMap.get('--fixed-block-size'));
      const percentageInlineSize = parseNumber(child.styleMap.get('--percentage-inline-size'));
      const percentageBlockSize = parseNumber(child.styleMap.get('--percentage-block-size'));
      return child.layoutNextFragment({
        availableInlineSize,
        availableBlockSize,
        fixedInlineSize,
        fixedBlockSize,
        percentageInlineSize,
        percentageBlockSize,
      });
    }));

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
