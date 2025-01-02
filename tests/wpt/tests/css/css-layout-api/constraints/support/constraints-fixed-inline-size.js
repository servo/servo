registerLayout('test', class {
  async intrinsicSizes() {}
  async layout(children, edges, constraints, styleMap) {
    if (constraints.fixedInlineSize !== 100)
      return {autoBlockSize: 0};

    return {autoBlockSize: 100};
  }
});
