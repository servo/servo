/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

function _defineProperty(obj, key, value) { if (key in obj) { Object.defineProperty(obj, key, { value: value, enumerable: true, configurable: true, writable: true }); } else { obj[key] = value; } return obj; }

function endOfRange(r) {
  return 'count' in r ? r.begin + r.count : r.end;
}

function* rangeAsIterator(r) {
  for (let i = r.begin; i < endOfRange(r); ++i) {
    yield i;
  }
}

export class SubresourceRange {
  constructor(subresources) {
    _defineProperty(this, "mipRange", void 0);

    _defineProperty(this, "sliceRange", void 0);

    this.mipRange = {
      begin: subresources.mipRange.begin,
      end: endOfRange(subresources.mipRange)
    };
    this.sliceRange = {
      begin: subresources.sliceRange.begin,
      end: endOfRange(subresources.sliceRange)
    };
  }

  *each() {
    for (let level = this.mipRange.begin; level < this.mipRange.end; ++level) {
      for (let slice = this.sliceRange.begin; slice < this.sliceRange.end; ++slice) {
        yield {
          level,
          slice
        };
      }
    }
  }

  *mipLevels() {
    for (let level = this.mipRange.begin; level < this.mipRange.end; ++level) {
      yield {
        level,
        slices: rangeAsIterator(this.sliceRange)
      };
    }
  }

}
//# sourceMappingURL=subresource.js.map