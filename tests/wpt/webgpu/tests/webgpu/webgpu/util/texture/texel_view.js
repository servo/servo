/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { assert, memcpy } from '../../../common/util/util.js';
import { kTextureFormatInfo } from '../../format_info.js';
import { generatePrettyTable } from '../pretty_diff_tables.js';
import { reifyExtent3D, reifyOrigin3D } from '../unions.js';

import { fullSubrectCoordinates } from './base.js';
import { kTexelRepresentationInfo, makeClampToRange } from './texel_data.js';

/** Function taking some x,y,z coordinates and returning `Readonly<T>`. */

/**
 * Wrapper to view various representations of texture data in other ways. E.g., can:
 * - Provide a mapped buffer, containing copied texture data, and read color values.
 * - Provide a function that generates color values by coordinate, and convert to ULPs-from-zero.
 *
 * MAINTENANCE_TODO: Would need some refactoring to support block formats, which could be partially
 * supported if useful.
 */
export class TexelView {
  /** The GPUTextureFormat of the TexelView. */

  /** Generates the bytes for the texel at the given coordinates. */

  /** Generates the ULPs-from-zero for the texel at the given coordinates. */

  /** Generates the color for the texel at the given coordinates. */

  constructor(format, { bytes, ulpFromZero, color }) {
    this.format = format;
    this.bytes = bytes;
    this.ulpFromZero = ulpFromZero;
    this.color = color;
  }

  /**
   * Produces a TexelView from "linear image data", i.e. the `writeTexture` format. Takes a
   * reference to the input `subrectData`, so any changes to it will be visible in the TexelView.
   */
  static fromTextureDataByReference(
    format,
    subrectData,
    { bytesPerRow, rowsPerImage, subrectOrigin, subrectSize }
  ) {
    const origin = reifyOrigin3D(subrectOrigin);
    const size = reifyExtent3D(subrectSize);

    const info = kTextureFormatInfo[format];
    assert(info.blockWidth === 1 && info.blockHeight === 1, 'unimplemented for block formats');

    return TexelView.fromTexelsAsBytes(format, coords => {
      assert(
        coords.x >= origin.x &&
          coords.y >= origin.y &&
          coords.z >= origin.z &&
          coords.x < origin.x + size.width &&
          coords.y < origin.y + size.height &&
          coords.z < origin.z + size.depthOrArrayLayers,
        () => `coordinate (${coords.x},${coords.y},${coords.z}) out of bounds`
      );

      const imageOffsetInRows = (coords.z - origin.z) * rowsPerImage;
      const rowOffset = (imageOffsetInRows + (coords.y - origin.y)) * bytesPerRow;
      const offset = rowOffset + (coords.x - origin.x) * info.bytesPerBlock;

      // MAINTENANCE_TODO: To support block formats, decode the block and then index into the result.
      return subrectData.subarray(offset, offset + info.bytesPerBlock);
    });
  }

  /** Produces a TexelView from a generator of bytes for individual texel blocks. */
  static fromTexelsAsBytes(format, generator) {
    const info = kTextureFormatInfo[format];
    assert(info.blockWidth === 1 && info.blockHeight === 1, 'unimplemented for block formats');

    const repr = kTexelRepresentationInfo[format];
    return new TexelView(format, {
      bytes: generator,
      ulpFromZero: coords => repr.bitsToULPFromZero(repr.unpackBits(generator(coords))),
      color: coords => repr.bitsToNumber(repr.unpackBits(generator(coords))),
    });
  }

  /** Produces a TexelView from a generator of numeric "color" values for each texel. */
  static fromTexelsAsColors(format, generator, { clampToFormatRange = false } = {}) {
    const info = kTextureFormatInfo[format];
    assert(info.blockWidth === 1 && info.blockHeight === 1, 'unimplemented for block formats');

    if (clampToFormatRange) {
      const applyClamp = makeClampToRange(format);
      const oldGenerator = generator;
      generator = coords => applyClamp(oldGenerator(coords));
    }

    const repr = kTexelRepresentationInfo[format];
    return new TexelView(format, {
      bytes: coords => new Uint8Array(repr.pack(repr.encode(generator(coords)))),
      ulpFromZero: coords => repr.bitsToULPFromZero(repr.numberToBits(generator(coords))),
      color: generator,
    });
  }

  /** Writes the contents of a TexelView as "linear image data", i.e. the `writeTexture` format. */
  writeTextureData(
    subrectData,
    { bytesPerRow, rowsPerImage, subrectOrigin: subrectOrigin_, subrectSize: subrectSize_ }
  ) {
    const subrectOrigin = reifyOrigin3D(subrectOrigin_);
    const subrectSize = reifyExtent3D(subrectSize_);

    const info = kTextureFormatInfo[this.format];
    assert(info.blockWidth === 1 && info.blockHeight === 1, 'unimplemented for block formats');

    for (let z = subrectOrigin.z; z < subrectOrigin.z + subrectSize.depthOrArrayLayers; ++z) {
      for (let y = subrectOrigin.y; y < subrectOrigin.y + subrectSize.height; ++y) {
        for (let x = subrectOrigin.x; x < subrectOrigin.x + subrectSize.width; ++x) {
          const start = (z * rowsPerImage + y) * bytesPerRow + x * info.bytesPerBlock;
          memcpy({ src: this.bytes({ x, y, z }) }, { dst: subrectData, start });
        }
      }
    }
  }

  /** Returns a pretty table string of the given coordinates and their values. */
  // MAINTENANCE_TODO: Unify some internal helpers with those in texture_ok.ts.
  toString(subrectOrigin, subrectSize) {
    const info = kTextureFormatInfo[this.format];
    const repr = kTexelRepresentationInfo[this.format];

    const integerSampleType = info.sampleType === 'uint' || info.sampleType === 'sint';
    const numberToString = integerSampleType ? n => n.toFixed() : n => n.toPrecision(6);

    const componentOrderStr = repr.componentOrder.join(',') + ':';
    const subrectCoords = [...fullSubrectCoordinates(subrectOrigin, subrectSize)];

    const printCoords = (function* () {
      yield* [' coords', '==', 'X,Y,Z:'];
      for (const coords of subrectCoords) yield `${coords.x},${coords.y},${coords.z}`;
    })();
    const printActualBytes = (function* (t) {
      yield* [' act. texel bytes (little-endian)', '==', '0x:'];
      for (const coords of subrectCoords) {
        yield Array.from(t.bytes(coords), b => b.toString(16).padStart(2, '0')).join(' ');
      }
    })(this);
    const printActualColors = (function* (t) {
      yield* [' act. colors', '==', componentOrderStr];
      for (const coords of subrectCoords) {
        const pixel = t.color(coords);
        yield `${repr.componentOrder.map(ch => numberToString(pixel[ch])).join(',')}`;
      }
    })(this);

    const opts = {
      fillToWidth: 120,
      numberToString,
    };
    return `${generatePrettyTable(opts, [printCoords, printActualBytes, printActualColors])}`;
  }
}
