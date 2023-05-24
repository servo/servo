/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/

function endOfRange(r) {
  return 'count' in r ? r.begin + r.count : r.end;
}

function* rangeAsIterator(r) {
  for (let i = r.begin; i < endOfRange(r); ++i) {
    yield i;
  }
}

/**
 * Represents a range of subresources of a single-plane texture:
 * a min/max mip level and min/max array layer.
 */
export class SubresourceRange {
  constructor(subresources) {
    this.mipRange = {
      begin: subresources.mipRange.begin,
      end: endOfRange(subresources.mipRange),
    };
    this.layerRange = {
      begin: subresources.layerRange.begin,
      end: endOfRange(subresources.layerRange),
    };
  }

  /**
   * Iterates over the "rectangle" of `{ level, layer }` pairs represented by the range.
   */
  *each() {
    for (let level = this.mipRange.begin; level < this.mipRange.end; ++level) {
      for (let layer = this.layerRange.begin; layer < this.layerRange.end; ++layer) {
        yield { level, layer };
      }
    }
  }

  /**
   * Iterates over the mip levels represented by the range, each level including an iterator
   * over the array layers at that level.
   */
  *mipLevels() {
    for (let level = this.mipRange.begin; level < this.mipRange.end; ++level) {
      yield {
        level,
        layers: rangeAsIterator(this.layerRange),
      };
    }
  }
}
