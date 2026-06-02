import {areArraysEqual} from '/common/arrays.js';

function parseNumber(value) {
  const num = parseInt(value.toString());
  if (isNaN(num)) return 0;
  return num;
}

registerLayout('test', class {
  static get inputProperties() {
    return [
      '--edges-inline-start-expected',
      '--edges-inline-end-expected',
      '--edges-block-start-expected',
      '--edges-block-end-expected',
    ];
  }

  async intrinsicSizes() {}
  async layout(children, edges, constraints, styleMap) {
    const actual = this.constructor.inputProperties.map(
      prop => parseNumber(styleMap.get(prop))
    );

    const expected = [
      edges.inlineStart,
      edges.inlineEnd,
      edges.blockStart,
      edges.blockEnd,
    ];

    if (!areArraysEqual(expected, actual)) {
      return {autoBlockSize: 0, childFragments: []};
    }

    return {autoBlockSize: 100, childFragment: []};
  }
});
