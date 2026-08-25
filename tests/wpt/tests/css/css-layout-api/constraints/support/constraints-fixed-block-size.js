registerLayout('test', class {
  static get inputProperties() {
    return ['--expected-block-size'];
  }

  async intrinsicSizes() {}
  async layout([child], edges, constraints, styleMap) {
    let childFixedInlineSize = 0;
    let childFixedBlockSize = 0;
    if (constraints.fixedBlockSize === JSON.parse(styleMap.get('--expected-block-size'))) {
      childFixedInlineSize = 100;
      childFixedBlockSize = 100;
    }

    const childFragments = [await child.layoutNextFragment({
      fixedInlineSize: childFixedInlineSize,
      fixedBlockSize: childFixedBlockSize,
    })];

    return {childFragments};
  }
});
