/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, unreachable } from '../../../common/util/util.js';import { kValue } from '../constants.js';
import {
  assertInIntegerRange,
  float32ToFloatBits,
  float32ToFloat16Bits,
  floatAsNormalizedInteger,
  gammaCompress,
  gammaDecompress,
  normalizedIntegerAsFloat,
  packRGB9E5UFloat,
  floatBitsToNumber,
  float16BitsToFloat32,
  floatBitsToNormalULPFromZero,
  kFloat32Format,
  kFloat16Format,
  kUFloat9e5Format,
  numberToFloat32Bits,
  float32BitsToNumber,
  numberToFloatBits,
  ufloatM9E5BitsToNumber } from
'../conversion.js';
import { clamp, signExtend } from '../math.js';

/** A component of a texture format: R, G, B, A, Depth, or Stencil. */
export let TexelComponent = /*#__PURE__*/function (TexelComponent) {TexelComponent["R"] = "R";TexelComponent["G"] = "G";TexelComponent["B"] = "B";TexelComponent["A"] = "A";TexelComponent["Depth"] = "Depth";TexelComponent["Stencil"] = "Stencil";return TexelComponent;}({});








/** Arbitrary data, per component of a texel format. */


/** How a component is encoded in its bit range of a texel format. */


/**
 * Maps component values to component values
 * @param {PerTexelComponent<number>} components - The input components.
 * @returns {PerTexelComponent<number>} The new output components.
 */


/**
 * Packs component values as an ArrayBuffer
 * @param {PerTexelComponent<number>} components - The input components.
 * @returns {ArrayBuffer} The packed data.
 */


/** Unpacks component values from a Uint8Array */


/**
 * Create a PerTexelComponent object filled with the same value for all components.
 * @param {TexelComponent[]} components - The component names.
 * @param {T} value - The value to assign to each component.
 * @returns {PerTexelComponent<T>}
 */
function makePerTexelComponent(components, value) {
  const values = {};
  for (const c of components) {
    values[c] = value;
  }
  return values;
}

/**
 * Create a function which applies clones a `PerTexelComponent<number>` and then applies the
 * function `fn` to each component of `components`.
 * @param {(value: number) => number} fn - The mapping function to apply to component values.
 * @param {TexelComponent[]} components - The component names.
 * @returns {ComponentMapFn} The map function which clones the input component values, and applies
 *                           `fn` to each of component of `components`.
 */
function applyEach(
fn,
components)
{
  return (values) => {
    values = Object.assign({}, values);
    for (const c of components) {
      assert(values[c] !== undefined);
      values[c] = fn(values[c], c);
    }
    return values;
  };
}

/**
 * A `ComponentMapFn` for encoding sRGB.
 * @param {PerTexelComponent<number>} components - The input component values.
 * @returns {TexelComponent<number>} Gamma-compressed copy of `components`.
 */
const encodeSRGB = (components) => {
  assert(
    components.R !== undefined && components.G !== undefined && components.B !== undefined,
    'sRGB requires all of R, G, and B components'
  );
  return applyEach(gammaCompress, kRGB)(components);
};

/**
 * A `ComponentMapFn` for decoding sRGB.
 * @param {PerTexelComponent<number>} components - The input component values.
 * @returns {TexelComponent<number>} Gamma-decompressed copy of `components`.
 */
const decodeSRGB = (components) => {
  components = Object.assign({}, components);
  assert(
    components.R !== undefined && components.G !== undefined && components.B !== undefined,
    'sRGB requires all of R, G, and B components'
  );
  return applyEach(gammaDecompress, kRGB)(components);
};

/**
 * Makes a `ComponentMapFn` for clamping values to the specified range.
 */
export function makeClampToRange(format) {
  const repr = kTexelRepresentationInfo[format];
  assert(repr.numericRange !== null, 'Format has unknown numericRange');
  const perComponentRanges = repr.numericRange;
  const range = repr.numericRange;

  return applyEach((x, component) => {
    const perComponentRange = perComponentRanges[component];
    return clamp(x, perComponentRange ? perComponentRange : range);
  }, repr.componentOrder);
}

// MAINTENANCE_TODO: Look into exposing this map to the test fixture so that it can be GCed at the
// end of each test group. That would allow for caching of larger buffers (though it's unclear how
// ofter larger buffers are used by packComponents.)
const smallComponentDataViews = new Map();
function getComponentDataView(byteLength) {
  if (byteLength > 32) {
    const buffer = new ArrayBuffer(byteLength);
    return new DataView(buffer);
  }
  let dataView = smallComponentDataViews.get(byteLength);
  if (!dataView) {
    const buffer = new ArrayBuffer(byteLength);
    dataView = new DataView(buffer);
    smallComponentDataViews.set(byteLength, dataView);
  }
  return dataView;
}

/**
 * Helper function to pack components as an ArrayBuffer.
 * @param {TexelComponent[]} componentOrder - The order of the component data.
 * @param {PerTexelComponent<number>} components - The input component values.
 * @param {number | PerTexelComponent<number>} bitLengths - The length in bits of each component.
 *   If a single number, all components are the same length, otherwise this is a dictionary of
 *   per-component bit lengths.
 * @param {ComponentDataType | PerTexelComponent<ComponentDataType>} componentDataTypes -
 *   The type of the data in `components`. If a single value, all components have the same value.
 *   Otherwise, this is a dictionary of per-component data types.
 * @returns {ArrayBuffer} The packed component data.
 */
function packComponents(
componentOrder,
components,
bitLengths,
componentDataTypes)
{
  let bitLengthMap;
  let totalBitLength;
  if (typeof bitLengths === 'number') {
    bitLengthMap = makePerTexelComponent(componentOrder, bitLengths);
    totalBitLength = bitLengths * componentOrder.length;
  } else {
    bitLengthMap = bitLengths;
    totalBitLength = Object.entries(bitLengthMap).reduce((acc, [, value]) => {
      assert(value !== undefined);
      return acc + value;
    }, 0);
  }
  assert(totalBitLength % 8 === 0);

  const componentDataTypeMap =
  typeof componentDataTypes === 'string' || componentDataTypes === null ?
  makePerTexelComponent(componentOrder, componentDataTypes) :
  componentDataTypes;

  const dataView = getComponentDataView(totalBitLength / 8);
  let bitOffset = 0;
  for (const c of componentOrder) {
    const value = components[c];
    const type = componentDataTypeMap[c];
    const bitLength = bitLengthMap[c];
    assert(value !== undefined);
    assert(type !== undefined);
    assert(bitLength !== undefined);

    const byteOffset = Math.floor(bitOffset / 8);
    const byteLength = Math.ceil(bitLength / 8);
    switch (type) {
      case 'uint':
      case 'unorm':
        if (byteOffset === bitOffset / 8 && byteLength === bitLength / 8) {
          switch (byteLength) {
            case 1:
              dataView.setUint8(byteOffset, value);
              break;
            case 2:
              dataView.setUint16(byteOffset, value, true);
              break;
            case 4:
              dataView.setUint32(byteOffset, value, true);
              break;
            default:
              unreachable();
          }
        } else {
          // Packed representations are all 32-bit and use Uint as the data type.
          // ex.) rg10b11float, rgb10a2unorm
          switch (dataView.byteLength) {
            case 4:{
                const currentValue = dataView.getUint32(0, true);

                let mask = 0xffffffff;
                const bitsToClearRight = bitOffset;
                const bitsToClearLeft = 32 - (bitLength + bitOffset);

                mask = mask >>> bitsToClearRight << bitsToClearRight;
                mask = mask << bitsToClearLeft >>> bitsToClearLeft;

                const newValue = currentValue & ~mask | value << bitOffset;

                dataView.setUint32(0, newValue, true);
                break;
              }
            default:
              unreachable();
          }
        }
        break;
      case 'sint':
      case 'snorm':
        assert(byteOffset === bitOffset / 8 && byteLength === bitLength / 8);
        switch (byteLength) {
          case 1:
            dataView.setInt8(byteOffset, value);
            break;
          case 2:
            dataView.setInt16(byteOffset, value, true);
            break;
          case 4:
            dataView.setInt32(byteOffset, value, true);
            break;
          default:
            unreachable();
        }
        break;
      case 'float':
        assert(byteOffset === bitOffset / 8 && byteLength === bitLength / 8);
        switch (byteLength) {
          case 4:
            dataView.setFloat32(byteOffset, value, true);
            break;
          default:
            unreachable();
        }
        break;
      case 'ufloat':
      case null:
        unreachable();
    }

    bitOffset += bitLength;
  }

  return dataView.buffer;
}

/**
 * Unpack substrings of bits from a Uint8Array, e.g. [8,8,8,8] or [9,9,9,5].
 */
function unpackComponentsBits(
componentOrder,
byteView,
bitLengths)
{
  const components = makePerTexelComponent(componentOrder, 0);

  let bitLengthMap;
  let totalBitLength;
  if (typeof bitLengths === 'number') {
    let index = 0;
    // Optimized cases for when the bit lengths are all a well aligned value.
    switch (bitLengths) {
      case 8:
        for (const c of componentOrder) {
          components[c] = byteView[index++];
        }
        return components;
      case 16:{
          const shortView = new Uint16Array(byteView.buffer, byteView.byteOffset);
          for (const c of componentOrder) {
            components[c] = shortView[index++];
          }
          return components;
        }
      case 32:{
          const longView = new Uint32Array(byteView.buffer, byteView.byteOffset);
          for (const c of componentOrder) {
            components[c] = longView[index++];
          }
          return components;
        }
    }

    bitLengthMap = makePerTexelComponent(componentOrder, bitLengths);
    totalBitLength = bitLengths * componentOrder.length;
  } else {
    bitLengthMap = bitLengths;
    totalBitLength = Object.entries(bitLengthMap).reduce((acc, [, value]) => {
      assert(value !== undefined);
      return acc + value;
    }, 0);
  }

  assert(totalBitLength % 8 === 0);

  const dataView = new DataView(byteView.buffer, byteView.byteOffset, byteView.byteLength);
  let bitOffset = 0;
  for (const c of componentOrder) {
    const bitLength = bitLengthMap[c];
    assert(bitLength !== undefined);

    let value;

    const byteOffset = Math.floor(bitOffset / 8);
    const byteLength = Math.ceil(bitLength / 8);
    if (byteOffset === bitOffset / 8 && byteLength === bitLength / 8) {
      switch (byteLength) {
        case 1:
          value = dataView.getUint8(byteOffset);
          break;
        case 2:
          value = dataView.getUint16(byteOffset, true);
          break;
        case 4:
          value = dataView.getUint32(byteOffset, true);
          break;
        default:
          unreachable();
      }
    } else {
      // Packed representations are all 32-bit and use Uint as the data type.
      // ex.) rg10b11float, rgb10a2unorm
      assert(dataView.byteLength === 4);
      const word = dataView.getUint32(0, true);
      value = word >>> bitOffset & (1 << bitLength) - 1;
    }

    bitOffset += bitLength;
    components[c] = value;
  }

  return components;
}

/**
 * Create an entry in `kTexelRepresentationInfo` for normalized integer texel data with constant
 * bitlength.
 * @param {TexelComponent[]} componentOrder - The order of the component data.
 * @param {number} bitLength - The number of bits in each component.
 * @param {{signed: boolean; sRGB: boolean}} opt - Boolean flags for `signed` and `sRGB`.
 */
function makeNormalizedInfo(
componentOrder,
bitLength,
opt)
{
  const encodeNonSRGB = applyEach(
    (n) => floatAsNormalizedInteger(n, bitLength, opt.signed),
    componentOrder
  );
  const decodeNonSRGB = applyEach(
    (n) => normalizedIntegerAsFloat(n, bitLength, opt.signed),
    componentOrder
  );

  const numberToBitsNonSRGB = applyEach(
    (n) => floatAsNormalizedInteger(n, bitLength, opt.signed),
    componentOrder
  );
  let bitsToNumberNonSRGB;
  if (opt.signed) {
    bitsToNumberNonSRGB = applyEach(
      (n) => normalizedIntegerAsFloat(signExtend(n, bitLength), bitLength, opt.signed),
      componentOrder
    );
  } else {
    bitsToNumberNonSRGB = applyEach(
      (n) => normalizedIntegerAsFloat(n, bitLength, opt.signed),
      componentOrder
    );
  }

  let encode;
  let decode;
  let numberToBits;
  let bitsToNumber;
  if (opt.sRGB) {
    encode = (components) => encodeNonSRGB(encodeSRGB(components));
    decode = (components) => decodeSRGB(decodeNonSRGB(components));
    numberToBits = (components) => numberToBitsNonSRGB(encodeSRGB(components));
    bitsToNumber = (components) => decodeSRGB(bitsToNumberNonSRGB(components));
  } else {
    encode = encodeNonSRGB;
    decode = decodeNonSRGB;
    numberToBits = numberToBitsNonSRGB;
    bitsToNumber = bitsToNumberNonSRGB;
  }

  let bitsToULPFromZero;
  if (opt.signed) {
    const maxValue = (1 << bitLength - 1) - 1; // e.g. 127 for snorm8
    bitsToULPFromZero = applyEach(
      (n) => Math.max(-maxValue, signExtend(n, bitLength)),
      componentOrder
    );
  } else {
    bitsToULPFromZero = (components) => components;
  }

  const dataType = opt.signed ? 'snorm' : 'unorm';
  const min = opt.signed ? -1 : 0;
  const max = 1;
  return {
    componentOrder,
    componentInfo: makePerTexelComponent(componentOrder, {
      dataType,
      bitLength
    }),
    encode,
    decode,
    pack: (components) =>
    packComponents(componentOrder, components, bitLength, dataType),
    unpackBits: (data) => unpackComponentsBits(componentOrder, data, bitLength),
    numberToBits,
    bitsToNumber,
    bitsToULPFromZero,
    numericRange: { min, max, finiteMin: min, finiteMax: max }
  };
}

/**
 * Create an entry in `kTexelRepresentationInfo` for integer texel data with constant bitlength.
 * @param {TexelComponent[]} componentOrder - The order of the component data.
 * @param {number} bitLength - The number of bits in each component.
 * @param {{signed: boolean}} opt - Boolean flag for `signed`.
 */
function makeIntegerInfo(
componentOrder,
bitLength,
opt)
{
  assert(bitLength <= 32);
  const min = opt.signed ? -(2 ** (bitLength - 1)) : 0;
  const max = opt.signed ? 2 ** (bitLength - 1) - 1 : 2 ** bitLength - 1;
  const numericRange = { min, max, finiteMin: min, finiteMax: max };
  const maxUnsignedValue = 2 ** bitLength;
  const encode = applyEach(
    (n) => (assertInIntegerRange(n, bitLength, opt.signed), n),
    componentOrder
  );
  const decode = applyEach(
    (n) => (assertInIntegerRange(n, bitLength, opt.signed), n),
    componentOrder
  );
  const bitsToNumber = applyEach((n) => {
    const decodedN = opt.signed ? n > numericRange.max ? n - maxUnsignedValue : n : n;
    assertInIntegerRange(decodedN, bitLength, opt.signed);
    return decodedN;
  }, componentOrder);

  let bitsToULPFromZero;
  if (opt.signed) {
    bitsToULPFromZero = applyEach((n) => signExtend(n, bitLength), componentOrder);
  } else {
    bitsToULPFromZero = (components) => components;
  }

  const dataType = opt.signed ? 'sint' : 'uint';
  const bitMask = (1 << bitLength) - 1;
  return {
    componentOrder,
    componentInfo: makePerTexelComponent(componentOrder, {
      dataType,
      bitLength
    }),
    encode,
    decode,
    pack: (components) =>
    packComponents(componentOrder, components, bitLength, dataType),
    unpackBits: (data) => unpackComponentsBits(componentOrder, data, bitLength),
    numberToBits: applyEach((v) => v & bitMask, componentOrder),
    bitsToNumber,
    bitsToULPFromZero,
    numericRange
  };
}

/**
 * Create an entry in `kTexelRepresentationInfo` for floating point texel data with constant
 * bitlength.
 * @param {TexelComponent[]} componentOrder - The order of the component data.
 * @param {number} bitLength - The number of bits in each component.
 */
function makeFloatInfo(
componentOrder,
bitLength,
{ restrictedDepth = false } = {})
{
  let encode;
  let numberToBits;
  let bitsToNumber;
  let bitsToULPFromZero;
  switch (bitLength) {
    case 32:
      if (restrictedDepth) {
        encode = applyEach((v) => {
          assert(v >= 0.0 && v <= 1.0, 'depth out of range');
          return new Float32Array([v])[0];
        }, componentOrder);
      } else {
        encode = applyEach((v) => new Float32Array([v])[0], componentOrder);
      }
      numberToBits = applyEach(numberToFloat32Bits, componentOrder);
      bitsToNumber = applyEach(float32BitsToNumber, componentOrder);
      bitsToULPFromZero = applyEach(
        (v) => floatBitsToNormalULPFromZero(v, kFloat32Format),
        componentOrder
      );
      break;
    case 16:
      if (restrictedDepth) {
        encode = applyEach((v) => {
          assert(v >= 0.0 && v <= 1.0, 'depth out of range');
          return float16BitsToFloat32(float32ToFloat16Bits(v));
        }, componentOrder);
      } else {
        encode = applyEach((v) => float16BitsToFloat32(float32ToFloat16Bits(v)), componentOrder);
      }
      numberToBits = applyEach(float32ToFloat16Bits, componentOrder);
      bitsToNumber = applyEach(float16BitsToFloat32, componentOrder);
      bitsToULPFromZero = applyEach(
        (v) => floatBitsToNormalULPFromZero(v, kFloat16Format),
        componentOrder
      );
      break;
    default:
      unreachable();
  }
  const decode = applyEach(identity, componentOrder);

  return {
    componentOrder,
    componentInfo: makePerTexelComponent(componentOrder, {
      dataType: 'float',
      bitLength
    }),
    encode,
    decode,
    pack: (components) => {
      switch (bitLength) {
        case 16:
          components = applyEach(float32ToFloat16Bits, componentOrder)(components);
          return packComponents(componentOrder, components, 16, 'uint');
        case 32:
          return packComponents(componentOrder, components, bitLength, 'float');
        default:
          unreachable();
      }
    },
    unpackBits: (data) => unpackComponentsBits(componentOrder, data, bitLength),
    numberToBits,
    bitsToNumber,
    bitsToULPFromZero,
    numericRange: restrictedDepth ?
    { min: 0, max: 1, finiteMin: 0, finiteMax: 1 } :
    {
      min: Number.NEGATIVE_INFINITY,
      max: Number.POSITIVE_INFINITY,
      finiteMin: bitLength === 32 ? kValue.f32.negative.min : kValue.f16.negative.min,
      finiteMax: bitLength === 32 ? kValue.f32.positive.max : kValue.f16.positive.max
    }
  };
}

const kR = [TexelComponent.R];
const kRG = [TexelComponent.R, TexelComponent.G];
const kRGB = [TexelComponent.R, TexelComponent.G, TexelComponent.B];
const kRGBA = [TexelComponent.R, TexelComponent.G, TexelComponent.B, TexelComponent.A];
const kBGRA = [TexelComponent.B, TexelComponent.G, TexelComponent.R, TexelComponent.A];

const identity = (n) => n;

const kFloat11Format = { signed: 0, exponentBits: 5, mantissaBits: 6, bias: 15 };
const kFloat10Format = { signed: 0, exponentBits: 5, mantissaBits: 5, bias: 15 };



















































export const kTexelRepresentationInfo =

{
  ...{
    'r8unorm': makeNormalizedInfo(kR, 8, { signed: false, sRGB: false }),
    'r8snorm': makeNormalizedInfo(kR, 8, { signed: true, sRGB: false }),
    'r8uint': makeIntegerInfo(kR, 8, { signed: false }),
    'r8sint': makeIntegerInfo(kR, 8, { signed: true }),
    'r16uint': makeIntegerInfo(kR, 16, { signed: false }),
    'r16sint': makeIntegerInfo(kR, 16, { signed: true }),
    'r16float': makeFloatInfo(kR, 16),
    'rg8unorm': makeNormalizedInfo(kRG, 8, { signed: false, sRGB: false }),
    'rg8snorm': makeNormalizedInfo(kRG, 8, { signed: true, sRGB: false }),
    'rg8uint': makeIntegerInfo(kRG, 8, { signed: false }),
    'rg8sint': makeIntegerInfo(kRG, 8, { signed: true }),
    'r32uint': makeIntegerInfo(kR, 32, { signed: false }),
    'r32sint': makeIntegerInfo(kR, 32, { signed: true }),
    'r32float': makeFloatInfo(kR, 32),
    'rg16uint': makeIntegerInfo(kRG, 16, { signed: false }),
    'rg16sint': makeIntegerInfo(kRG, 16, { signed: true }),
    'rg16float': makeFloatInfo(kRG, 16),
    'rgba8unorm': makeNormalizedInfo(kRGBA, 8, { signed: false, sRGB: false }),
    'rgba8unorm-srgb': makeNormalizedInfo(kRGBA, 8, { signed: false, sRGB: true }),
    'rgba8snorm': makeNormalizedInfo(kRGBA, 8, { signed: true, sRGB: false }),
    'rgba8uint': makeIntegerInfo(kRGBA, 8, { signed: false }),
    'rgba8sint': makeIntegerInfo(kRGBA, 8, { signed: true }),
    'bgra8unorm': makeNormalizedInfo(kBGRA, 8, { signed: false, sRGB: false }),
    'bgra8unorm-srgb': makeNormalizedInfo(kBGRA, 8, { signed: false, sRGB: true }),
    'rg32uint': makeIntegerInfo(kRG, 32, { signed: false }),
    'rg32sint': makeIntegerInfo(kRG, 32, { signed: true }),
    'rg32float': makeFloatInfo(kRG, 32),
    'rgba16uint': makeIntegerInfo(kRGBA, 16, { signed: false }),
    'rgba16sint': makeIntegerInfo(kRGBA, 16, { signed: true }),
    'rgba16float': makeFloatInfo(kRGBA, 16),
    'rgba32uint': makeIntegerInfo(kRGBA, 32, { signed: false }),
    'rgba32sint': makeIntegerInfo(kRGBA, 32, { signed: true }),
    'rgba32float': makeFloatInfo(kRGBA, 32)
  },
  ...{
    rgb10a2uint: {
      componentOrder: kRGBA,
      componentInfo: {
        R: { dataType: 'uint', bitLength: 10 },
        G: { dataType: 'uint', bitLength: 10 },
        B: { dataType: 'uint', bitLength: 10 },
        A: { dataType: 'uint', bitLength: 2 }
      },
      encode: (components) => {
        assertInIntegerRange(components.R, 10, false);
        assertInIntegerRange(components.G, 10, false);
        assertInIntegerRange(components.B, 10, false);
        assertInIntegerRange(components.A, 2, false);
        return components;
      },
      decode: (components) => {
        assertInIntegerRange(components.R, 10, false);
        assertInIntegerRange(components.G, 10, false);
        assertInIntegerRange(components.B, 10, false);
        assertInIntegerRange(components.A, 2, false);
        return components;
      },
      pack: (components) =>
      packComponents(
        kRGBA,
        components,
        {
          R: 10,
          G: 10,
          B: 10,
          A: 2
        },
        'uint'
      ),
      unpackBits: (data) =>
      unpackComponentsBits(kRGBA, data, { R: 10, G: 10, B: 10, A: 2 }),
      numberToBits: (components) => ({
        R: components.R & 0x3ff,
        G: components.G & 0x3ff,
        B: components.B & 0x3ff,
        A: components.A & 0x3
      }),
      bitsToNumber: (components) => {
        assertInIntegerRange(components.R, 10, false);
        assertInIntegerRange(components.G, 10, false);
        assertInIntegerRange(components.B, 10, false);
        assertInIntegerRange(components.A, 2, false);
        return components;
      },
      bitsToULPFromZero: (components) => components,
      numericRange: {
        R: { min: 0, max: 0x3ff, finiteMin: 0, finiteMax: 0x3ff },
        G: { min: 0, max: 0x3ff, finiteMin: 0, finiteMax: 0x3ff },
        B: { min: 0, max: 0x3ff, finiteMin: 0, finiteMax: 0x3ff },
        A: { min: 0, max: 0x3, finiteMin: 0, finiteMax: 0x3 }
      }
    },
    rgb10a2unorm: {
      componentOrder: kRGBA,
      componentInfo: {
        R: { dataType: 'unorm', bitLength: 10 },
        G: { dataType: 'unorm', bitLength: 10 },
        B: { dataType: 'unorm', bitLength: 10 },
        A: { dataType: 'unorm', bitLength: 2 }
      },
      encode: (components) => {
        return {
          R: floatAsNormalizedInteger(components.R ?? unreachable(), 10, false),
          G: floatAsNormalizedInteger(components.G ?? unreachable(), 10, false),
          B: floatAsNormalizedInteger(components.B ?? unreachable(), 10, false),
          A: floatAsNormalizedInteger(components.A ?? unreachable(), 2, false)
        };
      },
      decode: (components) => {
        return {
          R: normalizedIntegerAsFloat(components.R ?? unreachable(), 10, false),
          G: normalizedIntegerAsFloat(components.G ?? unreachable(), 10, false),
          B: normalizedIntegerAsFloat(components.B ?? unreachable(), 10, false),
          A: normalizedIntegerAsFloat(components.A ?? unreachable(), 2, false)
        };
      },
      pack: (components) =>
      packComponents(
        kRGBA,
        components,
        {
          R: 10,
          G: 10,
          B: 10,
          A: 2
        },
        'uint'
      ),
      unpackBits: (data) =>
      unpackComponentsBits(kRGBA, data, { R: 10, G: 10, B: 10, A: 2 }),
      numberToBits: (components) => ({
        R: floatAsNormalizedInteger(components.R ?? unreachable(), 10, false),
        G: floatAsNormalizedInteger(components.G ?? unreachable(), 10, false),
        B: floatAsNormalizedInteger(components.B ?? unreachable(), 10, false),
        A: floatAsNormalizedInteger(components.A ?? unreachable(), 2, false)
      }),
      bitsToNumber: (components) => ({
        R: normalizedIntegerAsFloat(components.R, 10, false),
        G: normalizedIntegerAsFloat(components.G, 10, false),
        B: normalizedIntegerAsFloat(components.B, 10, false),
        A: normalizedIntegerAsFloat(components.A, 2, false)
      }),
      bitsToULPFromZero: (components) => components,
      numericRange: { min: 0, max: 1, finiteMin: 0, finiteMax: 1 }
    },
    rg11b10ufloat: {
      componentOrder: kRGB,
      encode: applyEach(identity, kRGB),
      decode: applyEach(identity, kRGB),
      componentInfo: {
        R: { dataType: 'ufloat', bitLength: 11 },
        G: { dataType: 'ufloat', bitLength: 11 },
        B: { dataType: 'ufloat', bitLength: 10 }
      },
      pack: (components) => {
        const componentsBits = {
          R: float32ToFloatBits(components.R ?? unreachable(), 0, 5, 6, 15),
          G: float32ToFloatBits(components.G ?? unreachable(), 0, 5, 6, 15),
          B: float32ToFloatBits(components.B ?? unreachable(), 0, 5, 5, 15)
        };
        return packComponents(
          kRGB,
          componentsBits,
          {
            R: 11,
            G: 11,
            B: 10
          },
          'uint'
        );
      },
      unpackBits: (data) => unpackComponentsBits(kRGB, data, { R: 11, G: 11, B: 10 }),
      numberToBits: (components) => ({
        R: numberToFloatBits(components.R ?? unreachable(), kFloat11Format),
        G: numberToFloatBits(components.G ?? unreachable(), kFloat11Format),
        B: numberToFloatBits(components.B ?? unreachable(), kFloat10Format)
      }),
      bitsToNumber: (components) => ({
        R: floatBitsToNumber(components.R, kFloat11Format),
        G: floatBitsToNumber(components.G, kFloat11Format),
        B: floatBitsToNumber(components.B, kFloat10Format)
      }),
      bitsToULPFromZero: (components) => ({
        R: floatBitsToNormalULPFromZero(components.R, kFloat11Format),
        G: floatBitsToNormalULPFromZero(components.G, kFloat11Format),
        B: floatBitsToNormalULPFromZero(components.B, kFloat10Format)
      }),
      numericRange: {
        min: 0,
        max: Number.POSITIVE_INFINITY,
        finiteMin: 0,
        finiteMax: {
          R: floatBitsToNumber(0b111_1011_1111, kFloat11Format),
          G: floatBitsToNumber(0b111_1011_1111, kFloat11Format),
          B: floatBitsToNumber(0b11_1101_1111, kFloat10Format)
        }
      }
    },
    rgb9e5ufloat: {
      componentOrder: kRGB,
      componentInfo: makePerTexelComponent(kRGB, {
        dataType: 'ufloat',
        bitLength: -1 // Components don't really have a bitLength since the format is packed.
      }),
      encode: applyEach(identity, kRGB),
      decode: applyEach(identity, kRGB),
      pack: (components) =>
      new Uint32Array([
      packRGB9E5UFloat(
        components.R ?? unreachable(),
        components.G ?? unreachable(),
        components.B ?? unreachable()
      )]
      ).buffer,
      unpackBits: (data) => {
        const encoded = data[3] << 24 | data[2] << 16 | data[1] << 8 | data[0];
        const redMantissa = encoded >>> 0 & 0b111111111;
        const greenMantissa = encoded >>> 9 & 0b111111111;
        const blueMantissa = encoded >>> 18 & 0b111111111;
        const exponentSharedBits = (encoded >>> 27 & 0b11111) << 9;
        return {
          R: exponentSharedBits | redMantissa,
          G: exponentSharedBits | greenMantissa,
          B: exponentSharedBits | blueMantissa
        };
      },
      numberToBits: (components) => ({
        R: float32ToFloatBits(components.R ?? unreachable(), 0, 5, 9, 15),
        G: float32ToFloatBits(components.G ?? unreachable(), 0, 5, 9, 15),
        B: float32ToFloatBits(components.B ?? unreachable(), 0, 5, 9, 15)
      }),
      bitsToNumber: (components) => ({
        R: ufloatM9E5BitsToNumber(components.R, kUFloat9e5Format),
        G: ufloatM9E5BitsToNumber(components.G, kUFloat9e5Format),
        B: ufloatM9E5BitsToNumber(components.B, kUFloat9e5Format)
      }),
      bitsToULPFromZero: (components) => ({
        R: floatBitsToNormalULPFromZero(components.R, kUFloat9e5Format),
        G: floatBitsToNormalULPFromZero(components.G, kUFloat9e5Format),
        B: floatBitsToNormalULPFromZero(components.B, kUFloat9e5Format)
      }),
      numericRange: {
        min: 0,
        max: Number.POSITIVE_INFINITY,
        finiteMin: 0,
        finiteMax: ufloatM9E5BitsToNumber(0b11_1111_1111_1111, kUFloat9e5Format)
      }
    },
    depth32float: makeFloatInfo([TexelComponent.Depth], 32, { restrictedDepth: true }),
    depth16unorm: makeNormalizedInfo([TexelComponent.Depth], 16, { signed: false, sRGB: false }),
    depth24plus: {
      componentOrder: [TexelComponent.Depth],
      componentInfo: { Depth: { dataType: null, bitLength: 24 } },
      encode: applyEach(() => unreachable('depth24plus cannot be encoded'), [TexelComponent.Depth]),
      decode: applyEach(() => unreachable('depth24plus cannot be decoded'), [TexelComponent.Depth]),
      pack: () => unreachable('depth24plus data cannot be packed'),
      unpackBits: () => unreachable('depth24plus data cannot be unpacked'),
      numberToBits: () => unreachable('depth24plus has no representation'),
      bitsToNumber: () => unreachable('depth24plus has no representation'),
      bitsToULPFromZero: () => unreachable('depth24plus has no representation'),
      numericRange: { min: 0, max: 1, finiteMin: 0, finiteMax: 1 }
    },
    stencil8: makeIntegerInfo([TexelComponent.Stencil], 8, { signed: false }),
    'depth32float-stencil8': {
      componentOrder: [TexelComponent.Depth, TexelComponent.Stencil],
      componentInfo: {
        Depth: {
          dataType: 'float',
          bitLength: 32
        },
        Stencil: {
          dataType: 'uint',
          bitLength: 8
        }
      },
      encode: (components) => {
        assert(components.Stencil !== undefined);
        assertInIntegerRange(components.Stencil, 8, false);
        return components;
      },
      decode: (components) => {
        assert(components.Stencil !== undefined);
        assertInIntegerRange(components.Stencil, 8, false);
        return components;
      },
      pack: () => unreachable('depth32float-stencil8 data cannot be packed'),
      unpackBits: () => unreachable('depth32float-stencil8 data cannot be unpacked'),
      numberToBits: () => unreachable('not implemented'),
      bitsToNumber: () => unreachable('not implemented'),
      bitsToULPFromZero: () => unreachable('not implemented'),
      numericRange: null
    },
    'depth24plus-stencil8': {
      componentOrder: [TexelComponent.Depth, TexelComponent.Stencil],
      componentInfo: {
        Depth: {
          dataType: null,
          bitLength: 24
        },
        Stencil: {
          dataType: 'uint',
          bitLength: 8
        }
      },
      encode: (components) => {
        assert(components.Depth === undefined, 'depth24plus cannot be encoded');
        assert(components.Stencil !== undefined);
        assertInIntegerRange(components.Stencil, 8, false);
        return components;
      },
      decode: (components) => {
        assert(components.Depth === undefined, 'depth24plus cannot be decoded');
        assert(components.Stencil !== undefined);
        assertInIntegerRange(components.Stencil, 8, false);
        return components;
      },
      pack: () => unreachable('depth24plus-stencil8 data cannot be packed'),
      unpackBits: () => unreachable('depth24plus-stencil8 data cannot be unpacked'),
      numberToBits: () => unreachable('depth24plus-stencil8 has no representation'),
      bitsToNumber: () => unreachable('depth24plus-stencil8 has no representation'),
      bitsToULPFromZero: () => unreachable('depth24plus-stencil8 has no representation'),
      numericRange: null
    }
  }
};

/**
 * Get the `ComponentDataType` for a format. All components must have the same type.
 * @param {UncompressedTextureFormat} format - The input format.
 * @returns {ComponentDataType} The data of the components.
 */
export function getSingleDataType(format) {
  const infos = Object.values(kTexelRepresentationInfo[format].componentInfo);
  assert(infos.length > 0);
  return infos.reduce((acc, cur) => {
    assert(cur !== undefined);
    assert(acc === undefined || acc === cur.dataType);
    return cur.dataType;
  }, infos[0].dataType);
}

/**
 * Get traits for generating code to readback data from a component.
 * @param {ComponentDataType} dataType - The input component data type.
 * @returns A dictionary containing the respective `ReadbackTypedArray` and `shaderType`.
 */
export function getComponentReadbackTraits(dataType) {
  switch (dataType) {
    case 'ufloat':
    case 'float':
    case 'unorm':
    case 'snorm':
      return {
        ReadbackTypedArray: Float32Array,
        shaderType: 'f32'
      };
    case 'uint':
      return {
        ReadbackTypedArray: Uint32Array,
        shaderType: 'u32'
      };
    case 'sint':
      return {
        ReadbackTypedArray: Int32Array,
        shaderType: 'i32'
      };
    default:
      unreachable();
  }
}