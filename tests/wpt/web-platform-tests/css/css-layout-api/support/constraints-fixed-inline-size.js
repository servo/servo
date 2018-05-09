registerLayout('test', class {
  *intrinsicSizes() {}
  *layout(children, edges, constraints, styleMap) {
    if (constraints.fixedInlineSize !== 100)
      return {autoBlockSize: 0};

    return {autoBlockSize: 100};
  }
});
