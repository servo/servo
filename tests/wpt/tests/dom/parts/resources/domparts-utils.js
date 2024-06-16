function assertEqualParts(parts,partDescriptions,expectedParts,description) {
  assert_equals(parts.length,partDescriptions.length,`${description}: lengths differ`);
  for(let i=0;i<parts.length;++i) {
    assert_true(parts[i] instanceof Part,`${description}: not a Part`);
    assert_true(parts[i] instanceof window[partDescriptions[i].type],`${description}: index ${i} expected ${partDescriptions[i].type}`);
    // TODO(crbug.com/40271855): While developing alternative syntax, we aren't comparing the metadata:
    // assert_array_equals(parts[i].metadata,partDescriptions[i].metadata,`${description}: index ${i} wrong metadata`);
    if (expectedParts) {
      // TODO(crbug.com/40271855): While developing alternative syntax, we aren't comparing equality of the Part objects:
      // assert_equals(parts[i],expectedParts[i],`${description}: index ${i} object equality`);
      assert_equals(parts[i].root.getPartNode(i),parts[i].node || parts[i].previousSibling,'getPartNode() should return the same node as getParts().node/previousSibling');
    }
  }
}
