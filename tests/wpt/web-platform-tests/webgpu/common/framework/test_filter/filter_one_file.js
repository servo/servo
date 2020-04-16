/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { testSpecEquals } from '../id.js';
import { paramsEquals, paramsSupersets } from '../params_utils.js';

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

  definitelyOneFile() {
    return true;
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
    return filterTestGroup(spec.g, testcase => this.testMatches(testcase.test));
  }

  idIfSingle() {
    if (this.testPrefix.length !== 0) {
      return undefined;
    } // This is one whole spec file.


    return {
      spec: this.specId
    };
  }

  matches(id) {
    if (id.test === undefined) {
      return false;
    }

    return testSpecEquals(id.spec, this.specId) && this.testMatches(id.test);
  }

  testMatches(test) {
    return test.startsWith(this.testPrefix);
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
    return filterTestGroup(spec.g, testcase => this.caseMatches(testcase.test, testcase.params));
  }

  idIfSingle() {
    if (this.params !== null) {
      return undefined;
    } // This is one whole test.


    return {
      spec: this.specId,
      test: this.test
    };
  }

  matches(id) {
    if (id.test === undefined) {
      return false;
    }

    return testSpecEquals(id.spec, this.specId) && this.caseMatches(id.test, id.params);
  }

  caseMatches(test, params) {
    if (params === undefined) {
      return false;
    }

    return test === this.test && paramsSupersets(params, this.params);
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
    return filterTestGroup(spec.g, testcase => this.caseMatches(testcase.test, testcase.params));
  }

  idIfSingle() {
    // This is one single test case.
    return {
      spec: this.specId,
      test: this.test,
      params: this.params
    };
  }

  matches(id) {
    if (id.test === undefined || id.params === undefined) {
      return false;
    }

    return testSpecEquals(id.spec, this.specId) && this.caseMatches(id.test, id.params);
  }

  caseMatches(test, params) {
    return test === this.test && paramsEquals(params, this.params);
  }

}
//# sourceMappingURL=filter_one_file.js.map