registerLayout('test', class {
  static get childInputProperties() {
    return [
      '--inline-offset',
      '--block-offset',
    ];
  }

  async intrinsicSizes() {}
  async layout(children, edges, constraints, styleMap) {
    const childFragments = await Promise.all(children.map((child) => child.layoutNextFragment({})));

    for (let i = 0; i < children.length; i++) {
      childFragments[i].inlineOffset = parseInt(children[i].styleMap.get('--inline-offset').toString());
      childFragments[i].blockOffset = parseInt(children[i].styleMap.get('--block-offset').toString());
    }

    return {autoBlockSize: 0, childFragments};
  }
});
