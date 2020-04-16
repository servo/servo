/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { loadFilter } from './test_filter/load_filter.js';

function* concat(lists) {
  for (const specs of lists) {
    yield* specs;
  }
}

class DefaultTestFileLoader {
  async listing(suite) {
    return (await import(`../../${suite}/listing.js`)).listing;
  }

  import(path) {
    return import('../../' + path);
  }

}

export class TestLoader {
  constructor(fileLoader = new DefaultTestFileLoader()) {
    _defineProperty(this, "fileLoader", void 0);

    this.fileLoader = fileLoader;
  } // TODO: Test


  async loadTestsFromQuery(query) {
    return this.loadTests(new URLSearchParams(query).getAll('q'));
  } // TODO: Test
  // TODO: Probably should actually not exist at all, just use queries on cmd line too.


  async loadTestsFromCmdLine(filters) {
    return this.loadTests(filters);
  }

  async loadTests(filters) {
    const loads = filters.map(f => loadFilter(this.fileLoader, f));
    return concat((await Promise.all(loads)));
  }

}
//# sourceMappingURL=loader.js.map