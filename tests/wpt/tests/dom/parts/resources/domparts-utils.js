function assertEqualParts(parts,partDescriptions,expectedParts,description) {
  assert_equals(parts.length,partDescriptions.length,`${description}: lengths differ`);
  let nodePartIndx = 0, childNodePartIndx = 0;
  for(let i=0;i<parts.length;++i) {
    assert_true(parts[i] instanceof Part,`${description}: not a Part`);
    assert_true(parts[i] instanceof window[partDescriptions[i].type],`${description}: index ${i} expected ${partDescriptions[i].type}`);
    // TODO(crbug.com/40271855): While developing alternative syntax, we aren't comparing the metadata:
    // assert_array_equals(parts[i].metadata,partDescriptions[i].metadata,`${description}: index ${i} wrong metadata`);
    if (expectedParts) {
      // TODO(crbug.com/40271855): While developing alternative syntax, we aren't comparing equality of the Part objects:
      // assert_equals(parts[i],expectedParts[i],`${description}: index ${i} object equality`);
      if ('getNodePartNodes' in parts[i].root) {
        switch (partDescriptions[i].type) {
          case 'NodePart':
            assert_equals(parts[i].root.getNodePartNodes()[nodePartIndx],parts[i].node,`getNodePartNodes() indx ${nodePartIndx} should match node from NodePart`);
            nodePartIndx++;
            break;
          case 'ChildNodePart':
            assert_equals(parts[i].root.getChildNodePartNodes()[childNodePartIndx],parts[i].previousSibling,`getChildNodePartNodes() indx ${childNodePartIndx} should match previousSibling from ChildNodePart`);
            childNodePartIndx++;
            assert_equals(parts[i].root.getChildNodePartNodes()[childNodePartIndx],parts[i].nextSibling,`getChildNodePartNodes() indx ${childNodePartIndx} should match nextSibling from ChildNodePart`);
            childNodePartIndx++;
            break;
        }
      }
    }
  }
}
