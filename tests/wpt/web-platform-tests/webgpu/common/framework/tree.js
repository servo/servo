/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { compareQueries, Ordering } from './query/compare.js';
import { TestQueryMultiCase, TestQuerySingleCase, TestQueryMultiFile, TestQueryMultiTest } from './query/query.js';
import { stringifySingleParam } from './query/stringify_params.js';
import { assert } from './util/util.js'; // `loadTreeForQuery()` loads a TestTree for a given queryToLoad.
// The resulting tree is a linked-list all the way from `suite:*` to queryToLoad,
// and under queryToLoad is a tree containing every case matched by queryToLoad.
//
// `subqueriesToExpand` influences the `collapsible` flag on nodes in the resulting tree.
// A node is considered "collapsible" if none of the subqueriesToExpand is a StrictSubset
// of that node.
//
// In WebKit/Blink-style web_tests, an expectation file marks individual cts.html "variants" as
// "Failure", "Crash", etc.
// By passing in the list of expectations as the subqueriesToExpand, we can programmatically
// subdivide the cts.html "variants" list to be able to implement arbitrarily-fine suppressions
// (instead of having to suppress entire test files, which would lose a lot of coverage).
//
// `iterateCollapsedQueries()` produces the list of queries for the variants list.
//
// Though somewhat complicated, this system has important benefits:
//   - Avoids having to suppress entire test files, which would cause large test coverage loss.
//   - Minimizes the number of page loads needed for fine-grained suppressions.
//     (In the naive case, we could do one page load per test case - but the test suite would
//     take impossibly long to run.)
//   - Enables developers to put any number of tests in one file as appropriate, without worrying
//     about expectation granularity.

export class TestTree {
  constructor(root) {
    _defineProperty(this, "root", void 0);

    this.root = root;
  }

  iterateCollapsedQueries() {
    return TestTree.iterateSubtreeCollapsedQueries(this.root);
  }

  iterateLeaves() {
    return TestTree.iterateSubtreeLeaves(this.root);
  }

  toString() {
    return TestTree.subtreeToString('(root)', this.root, '');
  }

  static *iterateSubtreeCollapsedQueries(subtree) {
    for (const [, child] of subtree.children) {
      if ('children' in child && !child.collapsible) {
        yield* TestTree.iterateSubtreeCollapsedQueries(child);
      } else {
        yield child.query;
      }
    }
  }

  static *iterateSubtreeLeaves(subtree) {
    for (const [, child] of subtree.children) {
      if ('children' in child) {
        yield* TestTree.iterateSubtreeLeaves(child);
      } else {
        yield child;
      }
    }
  }

  static subtreeToString(name, tree, indent) {
    const collapsible = 'run' in tree ? '>' : tree.collapsible ? '+' : '-';
    let s = indent + `${collapsible} ${JSON.stringify(name)} => ` + `${tree.query}        ${JSON.stringify(tree.query)}`;

    if ('children' in tree) {
      if (tree.description !== undefined) {
        s += indent + `\n    | ${JSON.stringify(tree.description)}`;
      }

      for (const [name, child] of tree.children) {
        s += '\n' + TestTree.subtreeToString(name, child, indent + '  ');
      }
    }

    return s;
  }

} // TODO: Consider having subqueriesToExpand actually impact the depth-order of params in the tree.

export async function loadTreeForQuery(loader, queryToLoad, subqueriesToExpand) {
  const suite = queryToLoad.suite;
  const specs = await loader.listing(suite);
  const subqueriesToExpandEntries = Array.from(subqueriesToExpand.entries());
  const seenSubqueriesToExpand = new Array(subqueriesToExpand.length);
  seenSubqueriesToExpand.fill(false);

  const isCollapsible = subquery => subqueriesToExpandEntries.every(([i, toExpand]) => {
    const ordering = compareQueries(toExpand, subquery); // If toExpand == subquery, no expansion is needed (but it's still "seen").

    if (ordering === Ordering.Equal) seenSubqueriesToExpand[i] = true;
    return ordering !== Ordering.StrictSubset;
  }); // L0 = suite-level, e.g. suite:*
  // L1 =  file-level, e.g. suite:a,b:*
  // L2 =  test-level, e.g. suite:a,b:c,d:*
  // L3 =  case-level, e.g. suite:a,b:c,d:


  let foundCase = false; // L0 is suite:*

  const subtreeL0 = makeTreeForSuite(suite);
  isCollapsible(subtreeL0.query); // mark seenSubqueriesToExpand

  for (const entry of specs) {
    if (entry.file.length === 0 && 'readme' in entry) {
      // Suite-level readme.
      assert(subtreeL0.description === undefined);
      subtreeL0.description = entry.readme.trim();
      continue;
    }

    {
      const queryL1 = new TestQueryMultiFile(suite, entry.file);
      const orderingL1 = compareQueries(queryL1, queryToLoad);

      if (orderingL1 === Ordering.Unordered) {
        // File path is not matched by this query.
        continue;
      }
    }

    if ('readme' in entry) {
      // Entry is a README that is an ancestor or descendant of the query.
      // (It's included for display in the standalone runner.)
      // readmeSubtree is suite:a,b,*
      // (This is always going to dedup with a file path, if there are any test spec files under
      // the directory that has the README).
      const readmeSubtree = addSubtreeForDirPath(subtreeL0, entry.file);
      assert(readmeSubtree.description === undefined);
      readmeSubtree.description = entry.readme.trim();
      continue;
    } // Entry is a spec file.


    const spec = await loader.importSpecFile(queryToLoad.suite, entry.file);
    const description = spec.description.trim(); // subtreeL1 is suite:a,b:*

    const subtreeL1 = addSubtreeForFilePath(subtreeL0, entry.file, description, isCollapsible); // TODO: If tree generation gets too slow, avoid actually iterating the cases in a file
    // if there's no need to (based on the subqueriesToExpand).

    for (const t of spec.g.iterate()) {
      {
        const queryL3 = new TestQuerySingleCase(suite, entry.file, t.id.test, t.id.params);
        const orderingL3 = compareQueries(queryL3, queryToLoad);

        if (orderingL3 === Ordering.Unordered || orderingL3 === Ordering.StrictSuperset) {
          // Case is not matched by this query.
          continue;
        }
      } // subtreeL2 is suite:a,b:c,d:*

      const subtreeL2 = addSubtreeForTestPath(subtreeL1, t.id.test, isCollapsible); // Leaf for case is suite:a,b:c,d:x=1;y=2

      addLeafForCase(subtreeL2, t, isCollapsible);
      foundCase = true;
    }
  }

  const tree = new TestTree(subtreeL0);

  for (const [i, sq] of subqueriesToExpandEntries) {
    const seen = seenSubqueriesToExpand[i];
    assert(seen, `subqueriesToExpand entry did not match anything \
(can happen due to overlap with another subquery): ${sq.toString()}`);
  }

  assert(foundCase, 'Query does not match any cases'); // TODO: Contains lots of single-child subtrees. Consider cleaning those up (as postprocess?).

  return tree;
}

function makeTreeForSuite(suite) {
  return {
    query: new TestQueryMultiFile(suite, []),
    children: new Map(),
    collapsible: false
  };
}

function addSubtreeForDirPath(tree, file) {
  const subqueryFile = []; // To start, tree is suite:*
  // This loop goes from that -> suite:a,* -> suite:a,b,*

  for (const part of file) {
    subqueryFile.push(part);
    tree = getOrInsertSubtree(part, tree, () => {
      const query = new TestQueryMultiFile(tree.query.suite, subqueryFile);
      return {
        query,
        collapsible: false
      };
    });
  }

  return tree;
}

function addSubtreeForFilePath(tree, file, description, checkCollapsible) {
  // To start, tree is suite:*
  // This goes from that -> suite:a,* -> suite:a,b,*
  tree = addSubtreeForDirPath(tree, file); // This goes from that -> suite:a,b:*

  const subtree = getOrInsertSubtree('', tree, () => {
    const query = new TestQueryMultiTest(tree.query.suite, tree.query.filePathParts, []);
    return {
      query,
      description,
      collapsible: checkCollapsible(query)
    };
  });
  return subtree;
}

function addSubtreeForTestPath(tree, test, isCollapsible) {
  const subqueryTest = []; // To start, tree is suite:a,b:*
  // This loop goes from that -> suite:a,b:c,* -> suite:a,b:c,d,*

  for (const part of test) {
    subqueryTest.push(part);
    tree = getOrInsertSubtree(part, tree, () => {
      const query = new TestQueryMultiTest(tree.query.suite, tree.query.filePathParts, subqueryTest);
      return {
        query,
        collapsible: isCollapsible(query)
      };
    });
  } // This goes from that -> suite:a,b:c,d:*


  return getOrInsertSubtree('', tree, () => {
    const query = new TestQueryMultiCase(tree.query.suite, tree.query.filePathParts, subqueryTest, {});
    return {
      query,
      collapsible: isCollapsible(query)
    };
  });
}

function addLeafForCase(tree, t, checkCollapsible) {
  const query = tree.query;
  let name = '';
  const subqueryParams = {}; // To start, tree is suite:a,b:c,d:*
  // This loop goes from that -> suite:a,b:c,d:x=1;* -> suite:a,b:c,d:x=1;y=2;*

  for (const [k, v] of Object.entries(t.id.params)) {
    name = stringifySingleParam(k, v);
    subqueryParams[k] = v;
    tree = getOrInsertSubtree(name, tree, () => {
      const subquery = new TestQueryMultiCase(query.suite, query.filePathParts, query.testPathParts, subqueryParams);
      return {
        query: subquery,
        collapsible: checkCollapsible(subquery)
      };
    });
  } // This goes from that -> suite:a,b:c,d:x=1;y=2


  const subquery = new TestQuerySingleCase(query.suite, query.filePathParts, query.testPathParts, subqueryParams);
  checkCollapsible(subquery); // mark seenSubqueriesToExpand

  insertLeaf(tree, subquery, t);
}

function getOrInsertSubtree(key, parent, createSubtree) {
  let v;
  const child = parent.children.get(key);

  if (child !== undefined) {
    assert('children' in child); // Make sure cached subtree is not actually a leaf

    v = child;
  } else {
    v = { ...createSubtree(),
      children: new Map()
    };
    parent.children.set(key, v);
  }

  return v;
}

function insertLeaf(parent, query, t) {
  const key = '';
  const leaf = {
    query,
    run: rec => t.run(rec)
  };
  assert(!parent.children.has(key));
  parent.children.set(key, leaf);
}
//# sourceMappingURL=tree.js.map