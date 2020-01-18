/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

export class FilterByGroup {
  constructor(suite, groupPrefix) {
    _defineProperty(this, "suite", void 0);

    _defineProperty(this, "specPathPrefix", void 0);

    this.suite = suite;
    this.specPathPrefix = groupPrefix;
  }

  matches(id) {
    return id.spec.suite === this.suite && this.pathMatches(id.spec.path);
  }

  async iterate(loader) {
    const specs = await loader.listing(this.suite);
    const entries = [];
    const suite = this.suite;

    for (const {
      path,
      description
    } of specs) {
      if (this.pathMatches(path)) {
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

  definitelyOneFile() {
    // FilterByGroup could always possibly match multiple files, because it represents a prefix,
    // e.g. "a:b" not "a:b:".
    return false;
  }

  idIfSingle() {
    // FilterByGroup could be one whole suite, but we only want whole files, tests, or cases.
    return undefined;
  }

  pathMatches(path) {
    return path.startsWith(this.specPathPrefix);
  }

}
//# sourceMappingURL=filter_by_group.js.map