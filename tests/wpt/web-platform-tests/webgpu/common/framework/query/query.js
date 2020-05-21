/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

import { assert } from '../util/util.js';
import { encodeURIComponentSelectively } from './encode_selectively.js';
import { kBigSeparator, kPathSeparator, kWildcard, kParamSeparator } from './separators.js';
import { stringifyPublicParams } from './stringify_params.js';
export class TestQueryMultiFile {
  constructor(suite, file) {
    _defineProperty(this, "isMultiFile", true);

    _defineProperty(this, "suite", void 0);

    _defineProperty(this, "filePathParts", void 0);

    this.suite = suite;
    this.filePathParts = [...file];
  }

  toString() {
    return encodeURIComponentSelectively(this.toStringHelper().join(kBigSeparator));
  }

  toHTML() {
    return this.toStringHelper().join(kBigSeparator + '<wbr>');
  }

  toStringHelper() {
    return [this.suite, [...this.filePathParts, kWildcard].join(kPathSeparator)];
  }

}
export class TestQueryMultiTest extends TestQueryMultiFile {
  constructor(suite, file, test) {
    super(suite, file);

    _defineProperty(this, "isMultiFile", false);

    _defineProperty(this, "isMultiTest", true);

    _defineProperty(this, "testPathParts", void 0);

    assert(file.length > 0, 'multi-test (or finer) query must have file-path');
    this.testPathParts = [...test];
  }

  toStringHelper() {
    return [this.suite, this.filePathParts.join(kPathSeparator), [...this.testPathParts, kWildcard].join(kPathSeparator)];
  }

}
export class TestQueryMultiCase extends TestQueryMultiTest {
  constructor(suite, file, test, params) {
    super(suite, file, test);

    _defineProperty(this, "isMultiTest", false);

    _defineProperty(this, "isMultiCase", true);

    _defineProperty(this, "params", void 0);

    assert(test.length > 0, 'multi-case (or finer) query must have test-path');
    this.params = { ...params
    };
  }

  toStringHelper() {
    const paramsParts = stringifyPublicParams(this.params);
    return [this.suite, this.filePathParts.join(kPathSeparator), this.testPathParts.join(kPathSeparator), [...paramsParts, kWildcard].join(kParamSeparator)];
  }

}
export class TestQuerySingleCase extends TestQueryMultiCase {
  constructor(...args) {
    super(...args);

    _defineProperty(this, "isMultiCase", false);
  }

  toStringHelper() {
    const paramsParts = stringifyPublicParams(this.params);
    return [this.suite, this.filePathParts.join(kPathSeparator), this.testPathParts.join(kPathSeparator), paramsParts.join(kParamSeparator)];
  }

}
//# sourceMappingURL=query.js.map