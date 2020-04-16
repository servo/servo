/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { Logger } from './logger.js';
import { makeFilter } from './test_filter/load_filter.js';
import { treeFromFilterResults } from './tree.js';

function makeQuerySplitterTree(caselist, expectationStrings) {
  const expectations = [];

  for (const e of expectationStrings) {
    const filter = makeFilter(e);
    const id = filter.idIfSingle();

    if (!id) {
      throw new Error('Can only handle expectations which cover one file, one test, or one case. ' + e);
    }

    expectations.push({
      id,
      line: e,
      seen: false
    });
  }

  const convertToQuerySplitterTree = (tree, name) => {
    const children = tree.children;
    let needsSplit = true;

    if (name !== undefined) {
      const filter = makeFilter(name);
      const moreThanOneFile = !filter.definitelyOneFile();
      const matchingExpectations = expectations.map(e => {
        const matches = filter.matches(e.id);
        if (matches) e.seen = true;
        return matches;
      });
      needsSplit = matchingExpectations.some(m => m) || moreThanOneFile;
    }

    const queryNode = {
      needsSplit
    };

    if (children) {
      queryNode.children = new Map();

      for (const [k, v] of children) {
        const subtree = convertToQuerySplitterTree(v, k);
        queryNode.children.set(k, subtree);
      }
    }

    return queryNode;
  };

  const log = new Logger();
  const tree = treeFromFilterResults(log, caselist.values());
  const queryTree = convertToQuerySplitterTree(tree);

  for (const e of expectations) {
    if (!e.seen) throw new Error('expectation had no effect: ' + e.line);
  }

  return queryTree;
} // Takes a TestFilterResultIterator enumerating every test case in the suite, and a list of
// expectation queries from a browser's expectations file. Creates a minimal list of queries
// (i.e. wpt variant lines) such that:
//
// - There is at least one query per spec file.
// - Each of those those input queries is in the output, so that it can have its own expectation.
//
// It does this by creating a tree from the list of cases (same tree as the standalone runner uses),
// then marking every node which is a parent of a node that matches an expectation.


export async function generateMinimalQueryList(caselist, expectationStrings) {
  const unsplitNodes = [];

  const findUnsplitNodes = (name, node) => {
    if (node === undefined) {
      return;
    }

    if (node.needsSplit && node.children) {
      for (const [k, v] of node.children) {
        findUnsplitNodes(k, v);
      }
    } else {
      unsplitNodes.push(name);
    }
  };

  const queryTree = makeQuerySplitterTree(caselist, expectationStrings);
  findUnsplitNodes('', queryTree);

  for (const exp of expectationStrings) {
    if (!unsplitNodes.some(name => name === exp)) {
      throw new Error('Something went wrong: all expectation strings should always appear exactly: ' + exp);
    }
  }

  return unsplitNodes;
}
//# sourceMappingURL=generate_minimal_query_list.js.map