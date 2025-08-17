function createDeepDiv(nest) {
  const x = document.createElement('div');
  if (nest > 0)
    x.appendChild(createDeepDiv(nest - 1));
  return x;
}

function createDeepComponent(nest) {
  // Creating a nested component where a node is re-distributed into a slot in
  // a despendant shadow tree
  const div = document.createElement('div');
  div.appendChild(document.createElement('slot'));
  div.appendChild(document.createElement('p'));
  if (nest > 0) {
    const shadowRoot = div.attachShadow({ mode: 'open' });
    shadowRoot.appendChild(createDeepComponent(nest - 1));
  }
  return div;
}

function createHostTree(hostChildren) {
  return createHostTreeWith({
    hostChildren,
    createChildFunction: () => document.createElement('div'),
  });
}

function createHostTreeWithDeepComponentChild(hostChildren) {
  return createHostTreeWith({
    hostChildren,
    createChildFunction: () => createDeepComponent(100),
  });
}

function GetDeepestFirstChild(firstChild) {
  // Assuming a shadow root's first child always exists, and it can be a host.
  // createDeepComponent constructs such a tree.
  if (!firstChild.shadowRoot)
    return firstChild;
  return GetDeepestFirstChild(firstChild.shadowRoot.firstChild);
}

function rotateChildren(parent) {
  // A tree structure will change, rotating children.
  const firstChild = parent.firstChild;
  firstChild.remove();
  parent.appendChild(firstChild);
}

function removeLastChildAndAppend(host) {
  // A tree structure won't change
  const lastChild = host.lastChild;
  lastChild.remove();
  host.appendChild(lastChild);
}

function createHostTreeWith({hostChildren, createChildFunction}) {
  const host = document.createElement('div');
  host.id = 'host';
  for (let i = 0; i < hostChildren; ++i) {
    const div = createChildFunction();
    host.appendChild(div);
  }

  const shadowRoot = host.attachShadow({ mode: 'open' });
  shadowRoot.appendChild(document.createElement('slot'));
  return host;
}

function runHostChildrenMutationThenGetDistribution(host, loop) {
  const slot = host.shadowRoot.querySelector('slot');
  for (let i = 0; i < loop; ++i) {
    const firstChild = host.firstChild;
    firstChild.remove();
    host.appendChild(firstChild);
    slot.assignedNodes({ flatten: true });
  }
}

function runHostChildrenMutationThenLayout(host, loop) {
  for (let i = 0; i < loop; ++i) {
    const firstChild = host.firstChild;
    firstChild.remove();
    host.appendChild(firstChild);
    PerfTestRunner.forceLayout();
  }
}

function runHostChildrenMutationAppendThenLayout(host, loop) {
  for (let i = 0; i < loop; ++i) {
    host.appendChild(document.createElement('div'));
    PerfTestRunner.forceLayout();
  }
}

function runHostChildrenMutationPrependThenLayout(host, loop) {
  for (let i = 0; i < loop; ++i) {
    host.insertBefore(document.createElement('div'), host.firstChild);
    PerfTestRunner.forceLayout();
  }
}
