/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { allowedTestNameCharacters } from '../allowed_characters.js';
import { FilterByGroup } from './filter_by_group.js';
import { FilterByParamsExact, FilterByParamsMatch, FilterByTestMatch } from './filter_one_file.js';
// Each filter is of one of the forms below (urlencoded).
export async function loadFilter(loader, filter) {
  const i1 = filter.indexOf(':');

  if (i1 === -1) {
    throw new Error('Test queries must fully specify their suite name (e.g. "cts:")');
  }

  const suite = filter.substring(0, i1);
  const i2 = filter.indexOf(':', i1 + 1);

  if (i2 === -1) {
    // - cts:
    // - cts:buf
    // - cts:buffers/
    // - cts:buffers/map
    const groupPrefix = filter.substring(i1 + 1);
    return new FilterByGroup(suite, groupPrefix).iterate(loader);
  }

  const path = filter.substring(i1 + 1, i2);
  const endOfTestName = new RegExp('[^' + allowedTestNameCharacters + ']');
  const i3sub = filter.substring(i2 + 1).search(endOfTestName);

  if (i3sub === -1) {
    // - cts:buffers/mapWriteAsync:
    // - cts:buffers/mapWriteAsync:b
    const testPrefix = filter.substring(i2 + 1);
    return new FilterByTestMatch({
      suite,
      path
    }, testPrefix).iterate(loader);
  }

  const i3 = i2 + 1 + i3sub;
  const test = filter.substring(i2 + 1, i3);
  const token = filter.charAt(i3);
  let params = null;

  if (i3 + 1 < filter.length) {
    params = JSON.parse(filter.substring(i3 + 1));
  }

  if (token === '~') {
    // - cts:buffers/mapWriteAsync:basic~
    // - cts:buffers/mapWriteAsync:basic~{}
    // - cts:buffers/mapWriteAsync:basic~{filter:"params"}
    return new FilterByParamsMatch({
      suite,
      path
    }, test, params).iterate(loader);
  } else if (token === '=') {
    // - cts:buffers/mapWriteAsync:basic=
    // - cts:buffers/mapWriteAsync:basic={}
    // - cts:buffers/mapWriteAsync:basic={exact:"params"}
    return new FilterByParamsExact({
      suite,
      path
    }, test, params).iterate(loader);
  } else {
    throw new Error("invalid character after test name; must be '~' or '='");
  }
}
//# sourceMappingURL=load_filter.js.map