/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert } from '../util/util.js';
import { parseQuery } from './query/parseQuery.js';

import { loadTreeForQuery } from './tree.js';

// A listing file, e.g. either of:
// - `src/webgpu/listing.ts` (which is dynamically computed, has a Promise<TestSuiteListing>)
// - `out/webgpu/listing.js` (which is pre-baked, has a TestSuiteListing)

// Base class for DefaultTestFileLoader and FakeTestFileLoader.
export class TestFileLoader extends EventTarget {
  async importSpecFile(suite, path) {
    const url = `${suite}/${path.join('/')}.spec.js`;
    this.dispatchEvent(new MessageEvent('import', { data: { url } }));

    const ret = await this.import(url);
    this.dispatchEvent(new MessageEvent('imported', { data: { url } }));

    return ret;
  }

  async loadTree(query, { subqueriesToExpand = [], maxChunkTime = Infinity } = {}) {
    const tree = await loadTreeForQuery(this, query, {
      subqueriesToExpand: subqueriesToExpand.map(s => {
        const q = parseQuery(s);
        assert(q.level >= 2, () => `subqueriesToExpand entries should not be multi-file:\n  ${q}`);
        return q;
      }),
      maxChunkTime,
    });
    this.dispatchEvent(new MessageEvent('finish'));
    return tree;
  }

  async loadCases(query) {
    const tree = await this.loadTree(query);
    return tree.iterateLeaves();
  }
}

export class DefaultTestFileLoader extends TestFileLoader {
  async listing(suite) {
    return (await import(`../../${suite}/listing.js`)).listing;
  }

  import(path) {
    return import(`../../${path}`);
  }
}
