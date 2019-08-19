/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export class FilterByGroup {
  constructor(suite, groupPrefix) {
    _defineProperty(this, "suite", void 0);

    _defineProperty(this, "groupPrefix", void 0);

    this.suite = suite;
    this.groupPrefix = groupPrefix;
  }

  matches(spec, testcase) {
    throw new Error('unimplemented');
  }

  async iterate(loader) {
    const specs = await loader.listing(this.suite);
    const entries = [];
    const suite = this.suite;

    for (const {
      path,
      description
    } of specs) {
      if (path.startsWith(this.groupPrefix)) {
        const isReadme = path === '' || path.endsWith('/');
        const spec = isReadme ? {
          description
        } : await loader.import(`${suite}/${path}.spec.js`);
        entries.push({
          id: {
            suite,
            path
          },
          spec
        });
      }
    }

    return entries;
  }

}
//# sourceMappingURL=filter_by_group.js.map