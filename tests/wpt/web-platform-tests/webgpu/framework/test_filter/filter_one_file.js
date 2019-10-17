/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { paramsEquals, paramsSupersets } from '../params/index.js';

class FilterOneFile {
  constructor(specId) {
    _defineProperty(this, "specId", void 0);

    this.specId = specId;
  }

  async iterate(loader) {
    const spec = await loader.import(`${this.specId.suite}/${this.specId.path}.spec.js`);
    return [{
      id: this.specId,
      spec: {
        description: spec.description,
        g: this.getCases(spec)
      }
    }];
  }

}

function filterTestGroup(group, filter) {
  return {
    *iterate(log) {
      for (const rc of group.iterate(log)) {
        if (filter(rc.id)) {
          yield rc;
        }
      }
    }

  };
}

export class FilterByTestMatch extends FilterOneFile {
  constructor(specId, testPrefix) {
    super(specId);

    _defineProperty(this, "testPrefix", void 0);

    this.testPrefix = testPrefix;
  }

  getCases(spec) {
    return filterTestGroup(spec.g, testcase => testcase.test.startsWith(this.testPrefix));
  }

}
export class FilterByParamsMatch extends FilterOneFile {
  constructor(specId, test, params) {
    super(specId);

    _defineProperty(this, "test", void 0);

    _defineProperty(this, "params", void 0);

    this.test = test;
    this.params = params;
  }

  getCases(spec) {
    return filterTestGroup(spec.g, testcase => testcase.test === this.test && paramsSupersets(testcase.params, this.params));
  }

}
export class FilterByParamsExact extends FilterOneFile {
  constructor(specId, test, params) {
    super(specId);

    _defineProperty(this, "test", void 0);

    _defineProperty(this, "params", void 0);

    this.test = test;
    this.params = params;
  }

  getCases(spec) {
    return filterTestGroup(spec.g, testcase => testcase.test === this.test && paramsEquals(testcase.params, this.params));
  }

}
//# sourceMappingURL=filter_one_file.js.map