(() => {
  // Creates width <section> elements surrounded (optionally) by ChildNodeParts.
  // Each <section> gets width children of the same kind, down to a depth of depth.
  // If useParts is false, no Parts are constructed. If useParts is true, and
  // chainParts is true, descendant Parts use ancestor Parts as their PartRoot.
  // If chainParts is false, all Parts have the DocumentPartRoot as their root.
  const createContent = (node, root, useParts, chainParts, width, depth, extra, at = 0) => {
    at++;
    for (let i=0; i<width; i++) {
      const s = document.createComment('start');
      const c = document.createElement('section');
      c.textContent = `${at}.${i}`;
      let extras = [];
      for(let e=0;e<extra;e++) {
        extras.push(Object.assign(document.createElement('span'),{classList:'extra'}));
      }
      if (i==0) {
        const nodePartNode = document.createElement('section');
        nodePartNode.textContent = 'nodepart';
        extras.push(nodePartNode);
        if (useParts) {
          new NodePart(root, nodePartNode);
        }
      }
      const e = document.createComment('e');
      node.append(s, c, ...extras, e);
      let nextLevelRoot = root;
      if (useParts) {
        const newPart = new ChildNodePart(root, s, e);
        if (chainParts) {
          nextLevelRoot = newPart;
        }
      }
      if (at < depth) {
        createContent(c, nextLevelRoot, useParts, chainParts, width, depth, extra, at);
      }
    }
  }

  const createTemplateWrapper = () => {
    const template = document.createElement('template').content;
    const wrapper = template.appendChild(document.createElement('div'));
    return {template,wrapper};
  }
  let commentContent,partsFlatContent,partsNestedContent;
  let contentCreated = false;
  function createAllContent(width, depth, extra) {
    contentCreated = true;
    commentContent = createTemplateWrapper();
    createContent(commentContent.wrapper, commentContent.template.getPartRoot(), /*useParts*/false, 0, width, depth, extra);

    partsFlatContent = createTemplateWrapper();
    createContent(partsFlatContent.wrapper, partsFlatContent.template.getPartRoot(), /*useParts*/true, /*chainParts*/false, width, depth, extra);

    partsNestedContent = createTemplateWrapper();
    createContent(partsNestedContent.wrapper, partsNestedContent.template.getPartRoot(), /*useParts*/true, /*chainParts*/true, width, depth, extra);
  }


  const errorCheck = (container) => {
    if (!contentCreated) {
      throw new Error('content must be created with createAllContent()');
    }
    if (!container.isConnected || container.ownerDocument != document) {
      throw new Error('container must be in the document');
    }
    if (container.childNodes.length !== 0) {
      throw new Error('container must be empty');
    }
    if (document.getPartRoot().getParts().length !== 0) {
      throw new Error('test needs to start with no attached parts');
    }
    return {container};
  }

  const recursiveGetParts = (root,level=1) => {
    let parts = Array.from(root.getParts());
    const thisParts = [...parts];
    for(let part of thisParts) {
      if (part.getParts) {
        parts.push(...recursiveGetParts(part,level+1));
      }
    }
    return parts;
  }

  const countNodes = (node) => {
    let c = 1;
    node.childNodes.forEach(child => {
      c += countNodes(child);
    });
    return c;
  }

  const countNodesAndParts = (state) => {
    const nodes = countNodes(state.container);
    const root = state.container.ownerDocument.getPartRoot();
    let parts,nodeParts,childNodeParts;
    if (!root.getParts().length) {
      partCount = state.parts.length;
      nodeParts = state.parts.filter(p => p instanceof FakeNodePart).length;
      childNodeParts = state.parts.filter(p => p instanceof FakeChildNodePart).length;
    } else {
      const parts = recursiveGetParts(root);
      partCount = parts.length;
      nodeParts = parts.filter(p => p instanceof NodePart).length;
      childNodeParts = parts.filter(p => p instanceof ChildNodePart).length;
    }
    return {nodes, partCount, nodeParts, childNodeParts};
  }

  class FakeChildNodePart {
    start = null;
    end = null;
    constructor(start, end) {
      this.start = start;
      this.end = end;
    }
  }
  class FakeNodePart {
    node = null;
    constructor(node) {
      this.node = node;
    }
  }

  const testList = [
    {
      test: "Raw, no parts",
      prepare: (container) => errorCheck(container),
      clone: (state) => {
        state.clone = document.importNode(commentContent.template, true);
      },
      append: (state) => {
        state.container.appendChild(state.clone);
      },
      getParts: (state) => {
        state.parts = [];
      },
    },
    {
      test: "Manual tree walk",
      prepare: (container) => {
        state = errorCheck(container);
        state.walker = document.createTreeWalker(document, 129 /* NodeFilter.SHOW_{ELEMENT|COMMENT} */);
        return state;
      },
      clone: (state) => {
        state.clone = document.importNode(commentContent.template, true);
      },
      append: (state) => {
        state.container.appendChild(state.clone);
      },
      getParts: (state) => {
        const parts = [];
        state.walker.currentNode = state.container;
        while (state.walker.nextNode()) {
          const node = state.walker.currentNode;
          if (node.nodeType === Node.COMMENT_NODE && node.textContent === 'start') {
            parts.push(new FakeChildNodePart(node,node.nextSibling.nextSibling));
          } else if (node.nodeType === Node.ELEMENT_NODE && node.firstChild?.nodeType === Node.TEXT_NODE && node.firstChild.textContent === 'nodepart') {
            parts.push(new FakeNodePart(node));
          }
        }
        state.parts = parts;
      },
    },
    {
      test: "Parts, flat",
      prepare: (container) => errorCheck(container),
      clone: (state) => {
        state.clone = partsFlatContent.template.getPartRoot().clone().rootContainer;
      },
      append: (state) => {
        state.container.appendChild(state.clone);
      },
      getParts: (state) => {
        state.parts = document.getPartRoot().getParts();
      },
    },
    {
      test: "Parts, nested",
      prepare: (container) => errorCheck(container),
      clone: (state) => {
        state.clone = partsNestedContent.template.getPartRoot().clone().rootContainer;
      },
      append: (state) => {
        state.container.appendChild(state.clone);
      },
      getParts: (state) => {
        state.parts = recursiveGetParts(document.getPartRoot());
      },
    },
  ];

  const runTest = (testCase, repeats, container) => {
    let cloneTime = 0;
    let appendTime = 0;
    let getPartsTime = 0;
    let state;
    for(let r=0;r<repeats;++r) {
      // Clear out the old parts and content:
      container.replaceChildren();
      document.getPartRoot().getParts();
      if (typeof window.GCController !== "undefined") {
        // PerfTestRunner is providing GC.
        PerfTestRunner.gc();
      } else if (self.gc) {
        // This requires --js-flags="--expose-gc" on the command line.
        self.gc();
      } else {
        PerfTestRunner.assert_true(false,'This test requires some form of GC access');
      }
      state = testCase.prepare(container);
      // Run the test
      const start = performance.now();
      testCase.clone(state);
      const cloneDone = performance.now();
      testCase.append(state);
      const appendDone = performance.now();
      testCase.getParts(state);
      const partsDone = performance.now();
      cloneTime += cloneDone - start;
      appendTime += appendDone - cloneDone;
      getPartsTime += partsDone - appendDone;
    }
    return {cloneTime, appendTime, getPartsTime, state};
  }

  const findTestCase = (testType) => {
    const indx = testList.findIndex(v => v.test == testType);
    PerfTestRunner.assert_true(indx >= 0,`Unable to find test for ${testType}`);
    return testList[indx];
  }

  const runPerfTest = (testType,metric) => {
    const width = 4;
    const depth = 4;
    const extra = 8;
    const repeats = 10;
    const container = document.createElement('div');
    container.style="display:none";
    document.body.appendChild(container);
    PerfTestRunner.measureValue({
      description: `This benchmark tests the ${metric} time for ${testType}, for the DOM Parts API`,
      unit: 'ms',
      setup: () => {
        createAllContent(width, depth, extra);
      },
      run: function() {
        const results = runTest(findTestCase(testType), repeats, container);
        return results[metric];
      },
      warmUpCount: 2,
      iterationCount: 30,
    });
  }

  const manualRunTest = (testType, repeats, container) => {
    return runTest(findTestCase(testType), repeats, container);
  }

  // Exposed functions:
  window.runPerfTest =runPerfTest;
  window.createAllContent = createAllContent;
  window.manualRunTest = manualRunTest;
  window.countNodesAndParts = countNodesAndParts;
})();
