/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { loadMetadataForSuite } from '../framework/metadata.js';import { globalTestConfig } from '../framework/test_config.js';
import { assert, now } from '../util/util.js';



import { comparePublicParamsPaths, compareQueries, Ordering } from './query/compare.js';
import {

  TestQueryMultiCase,
  TestQuerySingleCase,
  TestQueryMultiFile,
  TestQueryMultiTest } from
'./query/query.js';
import { kBigSeparator, kWildcard, kPathSeparator, kParamSeparator } from './query/separators.js';
import { stringifySingleParam } from './query/stringify_params.js';
import { StacklessError } from './util.js';

// `loadTreeForQuery()` loads a TestTree for a given queryToLoad.
// The resulting tree is a linked-list all the way from `suite:*` to queryToLoad,
// and under queryToLoad is a tree containing every case matched by queryToLoad.
//
// `subqueriesToExpand` influences the `collapsible` flag on nodes in the resulting tree.
// A node is considered "collapsible" if none of the subqueriesToExpand is a StrictSubset
// of that node.
//
// In WebKit/Blink-style web_tests, an expectation file marks individual cts.https.html "variants
// as "Failure", "Crash", etc. By passing in the list of expectations as the subqueriesToExpand,
// we can programmatically subdivide the cts.https.html "variants" list to be able to implement
// arbitrarily-fine suppressions (instead of having to suppress entire test files, which would
// lose a lot of coverage).
//
// `iterateCollapsedNodes()` produces the list of queries for the variants list.
//
// Though somewhat complicated, this system has important benefits:
//   - Avoids having to suppress entire test files, which would cause large test coverage loss.
//   - Minimizes the number of page loads needed for fine-grained suppressions.
//     (In the naive case, we could do one page load per test case - but the test suite would
//     take impossibly long to run.)
//   - Enables developers to put any number of tests in one file as appropriate, without worrying
//     about expectation granularity.





























/**
 * When iterating through "collapsed" tree nodes, indicates how many "query levels" to traverse
 * through before starting to collapse nodes.
 *
 * Corresponds with TestQueryLevel, but excludes 4 (SingleCase):
 * - 1 = MultiFile. Expands so every file is in the collapsed tree.
 * - 2 = MultiTest. Expands so every test is in the collapsed tree.
 * - 3 = MultiCase. Expands so every case is in the collapsed tree (i.e. collapsing disabled).
 */


export class TestTree {
  /**
   * The `queryToLoad` that this test tree was created for.
   * Test trees are always rooted at `suite:*`, but they only contain nodes that fit
   * within `forQuery`.
   *
   * This is used for `iterateCollapsedNodes` which only starts collapsing at the next
   * `TestQueryLevel` after `forQuery`.
   */



  constructor(forQuery, root) {
    this.forQuery = forQuery;
    this.root = root;
    assert(
      root.query.level === 1 && root.query.depthInLevel === 0,
      'TestTree root must be the root (suite:*)'
    );
  }

  static async create(
  forQuery,
  root,
  maxChunkTime)
  {
    const suite = forQuery.suite;

    let chunking = undefined;
    if (Number.isFinite(maxChunkTime)) {
      const metadata = loadMetadataForSuite(`./src/${suite}`);
      assert(metadata !== null, `metadata for ${suite} is missing, but maxChunkTime was requested`);
      chunking = { metadata, maxChunkTime };
    }
    await TestTree.propagateCounts(root, chunking);

    return new TestTree(forQuery, root);
  }

  /**
   * Iterate through the leaves of a version of the tree which has been pruned to exclude
   * subtrees which:
   * - are at a deeper `TestQueryLevel` than `this.forQuery`, and
   * - were not a `Ordering.StrictSubset` of any of the `subqueriesToExpand` during tree creation.
   */
  iterateCollapsedNodes({
    includeIntermediateNodes = false,
    includeEmptySubtrees = false,
    alwaysExpandThroughLevel







  }) {
    const expandThroughLevel = Math.max(this.forQuery.level, alwaysExpandThroughLevel);
    return TestTree.iterateSubtreeNodes(this.root, {
      includeIntermediateNodes,
      includeEmptySubtrees,
      expandThroughLevel
    });
  }

  iterateLeaves() {
    return TestTree.iterateSubtreeLeaves(this.root);
  }

  /**
   * Dissolve nodes which have only one child, e.g.:
   *   a,* { a,b,* { a,b:* { ... } } }
   * collapses down into:
   *   a,* { a,b:* { ... } }
   * which is less needlessly verbose when displaying the tree in the standalone runner.
   */
  dissolveSingleChildTrees() {
    const newRoot = dissolveSingleChildTrees(this.root);
    assert(newRoot === this.root);
  }

  toString() {
    return TestTree.subtreeToString('(root)', this.root, '');
  }

  static *iterateSubtreeNodes(
  subtree,
  opts)




  {
    if (opts.includeIntermediateNodes) {
      yield subtree;
    }

    for (const [, child] of subtree.children) {
      if ('children' in child) {
        // Is a subtree
        const collapsible = child.collapsible && child.query.level > opts.expandThroughLevel;
        if (child.children.size > 0 && !collapsible) {
          yield* TestTree.iterateSubtreeNodes(child, opts);
        } else if (child.children.size > 0 || opts.includeEmptySubtrees) {
          // Don't yield empty subtrees (e.g. files with no tests) unless includeEmptySubtrees
          yield child;
        }
      } else {
        // Is a leaf
        yield child;
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

  /** Propagate the subtreeTODOs/subtreeTests state upward from leaves to parent nodes. */
  static async propagateCounts(
  subtree,
  chunking)
  {
    subtree.subtreeCounts ??= { tests: 0, nodesWithTODO: 0, totalTimeMS: 0 };
    subtree.subcaseCount = 0;
    for (const [, child] of subtree.children) {
      if ('children' in child) {
        const counts = await TestTree.propagateCounts(child, chunking);
        subtree.subtreeCounts.tests += counts.tests;
        subtree.subtreeCounts.nodesWithTODO += counts.nodesWithTODO;
        subtree.subtreeCounts.totalTimeMS += counts.totalTimeMS;
        subtree.subcaseCount += counts.subcaseCount;
      } else {
        subtree.subcaseCount = child.subcaseCount;
      }
    }

    // If we're chunking based on a maxChunkTime, then at each
    // TestQueryMultiCase node of the tree we look at its total time. If the
    // total time is larger than the maxChunkTime, we set collapsible=false to
    // make sure it gets split up in the output. Note:
    // - TestQueryMultiTest and higher nodes are never set to collapsible anyway, so we ignore them.
    // - TestQuerySingleCase nodes can't be collapsed, so we ignore them.
    if (chunking && subtree.query instanceof TestQueryMultiCase) {
      const testLevelQuery = new TestQueryMultiCase(
        subtree.query.suite,
        subtree.query.filePathParts,
        subtree.query.testPathParts,
        {}
      ).toString();

      const metadata = chunking.metadata;

      const subcaseTiming = metadata[testLevelQuery]?.subcaseMS;
      if (subcaseTiming !== undefined) {
        const totalTiming = subcaseTiming * subtree.subcaseCount;
        subtree.subtreeCounts.totalTimeMS = totalTiming;
        if (totalTiming > chunking.maxChunkTime) {
          subtree.collapsible = false;
        }
      }
    }

    return { ...subtree.subtreeCounts, subcaseCount: subtree.subcaseCount ?? 0 };
  }

  /** Displays counts in the format `(Nodes with TODOs) / (Total test count)`. */
  static countsToString(tree) {
    if (tree.subtreeCounts) {
      return `${tree.subtreeCounts.nodesWithTODO} / ${tree.subtreeCounts.tests}`;
    } else {
      return '';
    }
  }

  static subtreeToString(name, tree, indent) {
    const collapsible = 'run' in tree ? '>' : tree.collapsible ? '+' : '-';
    let s =
    indent +
    `${collapsible} ${TestTree.countsToString(tree)} ${JSON.stringify(name)} => ${tree.query}`;
    if ('children' in tree) {
      if (tree.description !== undefined) {
        s += `\n${indent}  | ${JSON.stringify(tree.description)}`;
      }

      for (const [name, child] of tree.children) {
        s += '\n' + TestTree.subtreeToString(name, child, indent + '  ');
      }
    }
    return s;
  }
}

// MAINTENANCE_TODO: Consider having subqueriesToExpand actually impact the depth-order of params
// in the tree.
export async function loadTreeForQuery(
loader,
queryToLoad,
{
  subqueriesToExpand,
  maxChunkTime = Infinity
})
{
  const suite = queryToLoad.suite;
  const specs = await loader.listing(suite);

  const subqueriesToExpandEntries = Array.from(subqueriesToExpand.entries());
  const seenSubqueriesToExpand = new Array(subqueriesToExpand.length);
  seenSubqueriesToExpand.fill(false);

  const isCollapsible = (subquery) =>
  subqueriesToExpandEntries.every(([i, toExpand]) => {
    const ordering = compareQueries(toExpand, subquery);

    // If toExpand == subquery, no expansion is needed (but it's still "seen").
    if (ordering === Ordering.Equal) seenSubqueriesToExpand[i] = true;
    return ordering !== Ordering.StrictSubset;
  });

  // L0 = suite-level, e.g. suite:*
  // L1 =  file-level, e.g. suite:a,b:*
  // L2 =  test-level, e.g. suite:a,b:c,d:*
  // L3 =  case-level, e.g. suite:a,b:c,d:
  let foundCase = false;
  // L0 is suite:*
  const subtreeL0 = makeTreeForSuite(suite, isCollapsible);

  const imports_start = now();
  const pEntriesWithImports = []; // Promise<entry with importedSpec>[]
  for (const entry of specs) {
    if (entry.file.length === 0 && 'readme' in entry) {
      // Suite-level readme.
      setSubtreeDescriptionAndCountTODOs(subtreeL0, entry.readme);
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

    // We're going to be fetching+importing a bunch of things, so do it in async.
    const pEntryWithImport = (async () => {
      if ('readme' in entry) {
        return entry;
      } else {
        return {
          ...entry,
          importedSpec: await loader.importSpecFile(queryToLoad.suite, entry.file)
        };
      }
    })();

    const kForceSerialImporting = false;
    if (kForceSerialImporting) {
      await pEntryWithImport;
    }
    pEntriesWithImports.push(pEntryWithImport);
  }

  const entriesWithImports = await Promise.all(pEntriesWithImports);
  if (globalTestConfig.frameworkDebugLog) {
    const imported_time = performance.now() - imports_start;
    globalTestConfig.frameworkDebugLog(
      `Imported importedSpecFiles[${entriesWithImports.length}] in ${imported_time}ms.`
    );
  }

  for (const entry of entriesWithImports) {
    if ('readme' in entry) {
      // Entry is a README that is an ancestor or descendant of the query.
      // (It's included for display in the standalone runner.)

      // readmeSubtree is suite:a,b,*
      // (This is always going to dedup with a file path, if there are any test spec files under
      // the directory that has the README).
      const readmeSubtree = addSubtreeForDirPath(
        subtreeL0,
        entry.file,
        isCollapsible
      );
      setSubtreeDescriptionAndCountTODOs(readmeSubtree, entry.readme);
      continue;
    }

    // Entry is a spec file.
    const spec = entry.importedSpec;
    // subtreeL1 is suite:a,b:*
    const subtreeL1 = addSubtreeForFilePath(
      subtreeL0,
      entry.file,
      isCollapsible
    );
    setSubtreeDescriptionAndCountTODOs(subtreeL1, spec.description);

    let groupHasTests = false;
    for (const t of spec.g.iterate()) {
      groupHasTests = true;
      {
        const queryL2 = new TestQueryMultiCase(suite, entry.file, t.testPath, {});
        const orderingL2 = compareQueries(queryL2, queryToLoad);
        if (orderingL2 === Ordering.Unordered) {
          // Test path is not matched by this query.
          continue;
        }
      }

      // subtreeL2 is suite:a,b:c,d:*
      const subtreeL2 = addSubtreeForTestPath(
        subtreeL1,
        t.testPath,
        t.testCreationStack,
        isCollapsible
      );
      // This is 1 test. Set tests=1 then count TODOs.
      subtreeL2.subtreeCounts ??= { tests: 1, nodesWithTODO: 0, totalTimeMS: 0 };
      if (t.description) setSubtreeDescriptionAndCountTODOs(subtreeL2, t.description);

      let caseFilter = null;
      if ('params' in queryToLoad) {
        caseFilter = queryToLoad.params;
      }

      // MAINTENANCE_TODO: If tree generation gets too slow, avoid actually iterating the cases in a
      // file if there's no need to (based on the subqueriesToExpand).
      for (const c of t.iterate(caseFilter)) {
        // iterate() guarantees c's query is equal to or a subset of queryToLoad.

        if (queryToLoad instanceof TestQuerySingleCase) {
          // A subset is OK if it's TestQueryMultiCase, but for SingleCase it must match exactly.
          const ordering = comparePublicParamsPaths(c.id.params, queryToLoad.params);
          if (ordering !== Ordering.Equal) {
            continue;
          }
        }

        // Leaf for case is suite:a,b:c,d:x=1;y=2
        addLeafForCase(subtreeL2, c, isCollapsible);
        foundCase = true;
      }
    }
    if (!groupHasTests && !subtreeL1.subtreeCounts) {
      throw new StacklessError(
        `${subtreeL1.query} has no tests - it must have "TODO" in its description`
      );
    }
  }

  for (const [i, sq] of subqueriesToExpandEntries) {
    const subquerySeen = seenSubqueriesToExpand[i];
    if (!subquerySeen) {
      throw new StacklessError(
        `subqueriesToExpand entry did not match anything \
(could be wrong, or could be redundant with a previous subquery):\n  ${sq.toString()}`
      );
    }
  }
  assert(foundCase, `Query \`${queryToLoad.toString()}\` does not match any cases`);

  return TestTree.create(queryToLoad, subtreeL0, maxChunkTime);
}

function setSubtreeDescriptionAndCountTODOs(
subtree,
description)
{
  assert(subtree.description === undefined);
  subtree.description = description.trim();
  subtree.subtreeCounts ??= { tests: 0, nodesWithTODO: 0, totalTimeMS: 0 };
  if (subtree.description.indexOf('TODO') !== -1) {
    subtree.subtreeCounts.nodesWithTODO++;
  }
}

function makeTreeForSuite(
suite,
isCollapsible)
{
  const query = new TestQueryMultiFile(suite, []);
  return {
    readableRelativeName: suite + kBigSeparator,
    query,
    children: new Map(),
    collapsible: isCollapsible(query)
  };
}

function addSubtreeForDirPath(
tree,
file,
isCollapsible)
{
  const subqueryFile = [];
  // To start, tree is suite:*
  // This loop goes from that -> suite:a,* -> suite:a,b,*
  for (const part of file) {
    subqueryFile.push(part);
    tree = getOrInsertSubtree(part, tree, () => {
      const query = new TestQueryMultiFile(tree.query.suite, subqueryFile);
      return {
        readableRelativeName: part + kPathSeparator + kWildcard,
        query,
        collapsible: isCollapsible(query)
      };
    });
  }
  return tree;
}

function addSubtreeForFilePath(
tree,
file,
isCollapsible)
{
  // To start, tree is suite:*
  // This goes from that -> suite:a,* -> suite:a,b,*
  tree = addSubtreeForDirPath(tree, file, isCollapsible);
  // This goes from that -> suite:a,b:*
  const subtree = getOrInsertSubtree('', tree, () => {
    const query = new TestQueryMultiTest(tree.query.suite, tree.query.filePathParts, []);
    assert(file.length > 0, 'file path is empty');
    return {
      readableRelativeName: file[file.length - 1] + kBigSeparator + kWildcard,
      query,
      collapsible: isCollapsible(query)
    };
  });
  return subtree;
}

function addSubtreeForTestPath(
tree,
test,
testCreationStack,
isCollapsible)
{
  const subqueryTest = [];
  // To start, tree is suite:a,b:*
  // This loop goes from that -> suite:a,b:c,* -> suite:a,b:c,d,*
  for (const part of test) {
    subqueryTest.push(part);
    tree = getOrInsertSubtree(part, tree, () => {
      const query = new TestQueryMultiTest(
        tree.query.suite,
        tree.query.filePathParts,
        subqueryTest
      );
      return {
        readableRelativeName: part + kPathSeparator + kWildcard,
        query,
        collapsible: isCollapsible(query)
      };
    });
  }
  // This goes from that -> suite:a,b:c,d:*
  return getOrInsertSubtree('', tree, () => {
    const query = new TestQueryMultiCase(
      tree.query.suite,
      tree.query.filePathParts,
      subqueryTest,
      {}
    );
    assert(subqueryTest.length > 0, 'subqueryTest is empty');
    return {
      readableRelativeName: subqueryTest[subqueryTest.length - 1] + kBigSeparator + kWildcard,
      kWildcard,
      query,
      testCreationStack,
      collapsible: isCollapsible(query)
    };
  });
}

function addLeafForCase(
tree,
t,
checkCollapsible)
{
  const query = tree.query;
  let name = '';
  const subqueryParams = {};

  // To start, tree is suite:a,b:c,d:*
  // This loop goes from that -> suite:a,b:c,d:x=1;* -> suite:a,b:c,d:x=1;y=2;*
  for (const [k, v] of Object.entries(t.id.params)) {
    name = stringifySingleParam(k, v);
    subqueryParams[k] = v;

    tree = getOrInsertSubtree(name, tree, () => {
      const subquery = new TestQueryMultiCase(
        query.suite,
        query.filePathParts,
        query.testPathParts,
        subqueryParams
      );
      return {
        readableRelativeName: name + kParamSeparator + kWildcard,
        query: subquery,
        collapsible: checkCollapsible(subquery)
      };
    });
  }

  // This goes from that -> suite:a,b:c,d:x=1;y=2
  const subquery = new TestQuerySingleCase(
    query.suite,
    query.filePathParts,
    query.testPathParts,
    subqueryParams
  );
  checkCollapsible(subquery); // mark seenSubqueriesToExpand
  insertLeaf(tree, subquery, t);
}

function getOrInsertSubtree(
key,
parent,
createSubtree)
{
  let v;
  const child = parent.children.get(key);
  if (child !== undefined) {
    assert('children' in child); // Make sure cached subtree is not actually a leaf
    v = child;
  } else {
    v = { ...createSubtree(), children: new Map() };
    parent.children.set(key, v);
  }
  return v;
}

function insertLeaf(parent, query, t) {
  const leaf = {
    readableRelativeName: readableNameForCase(query),
    query,
    run: (rec, expectations) => t.run(rec, query, expectations || []),
    isUnimplemented: t.isUnimplemented,
    subcaseCount: t.computeSubcaseCount()
  };

  // This is a leaf (e.g. s:f:t:x=1;* -> s:f:t:x=1). The key is always ''.
  const key = '';
  assert(!parent.children.has(key), `Duplicate testcase: ${query}`);
  parent.children.set(key, leaf);
}

function dissolveSingleChildTrees(tree) {
  if ('children' in tree) {
    const shouldDissolveThisTree =
    tree.children.size === 1 && tree.query.depthInLevel !== 0 && tree.description === undefined;
    if (shouldDissolveThisTree) {
      // Loops exactly once
      for (const [, child] of tree.children) {
        // Recurse on child
        return dissolveSingleChildTrees(child);
      }
    }

    for (const [k, child] of tree.children) {
      // Recurse on each child
      const newChild = dissolveSingleChildTrees(child);
      if (newChild !== child) {
        tree.children.set(k, newChild);
      }
    }
  }
  return tree;
}

/** Generate a readable relative name for a case (used in standalone). */
function readableNameForCase(query) {
  const paramsKeys = Object.keys(query.params);
  if (paramsKeys.length === 0) {
    return query.testPathParts[query.testPathParts.length - 1] + kBigSeparator;
  } else {
    const lastKey = paramsKeys[paramsKeys.length - 1];
    return stringifySingleParam(lastKey, query.params[lastKey]);
  }
}