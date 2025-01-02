/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { assert, ErrorWithExtra, unreachable } from '../../../common/util/util.js';import { kTextureFormatInfo } from '../../format_info.js';
import { numbersApproximatelyEqual } from '../conversion.js';
import { generatePrettyTable, numericToStringBuilder } from '../pretty_diff_tables.js';
import { reifyExtent3D, reifyOrigin3D } from '../unions.js';

import { fullSubrectCoordinates } from './base.js';
import { getTextureSubCopyLayout } from './layout.js';
import { kTexelRepresentationInfo } from './texel_data.js';
import { TexelView } from './texel_view.js';



/** Threshold options for comparing texels of different formats (norm/float/int). */




























function makeTexelViewComparer(
format,
{ actTexelView, expTexelView },
opts)
{
  const {
    maxIntDiff = 0,
    maxFractionalDiff,
    maxDiffULPsForNormFormat,
    maxDiffULPsForFloatFormat
  } = opts;

  assert(maxIntDiff >= 0, 'threshold must be non-negative');
  if (maxFractionalDiff !== undefined) {
    assert(maxFractionalDiff >= 0, 'threshold must be non-negative');
  }
  if (maxDiffULPsForFloatFormat !== undefined) {
    assert(maxDiffULPsForFloatFormat >= 0, 'threshold must be non-negative');
  }
  if (maxDiffULPsForNormFormat !== undefined) {
    assert(maxDiffULPsForNormFormat >= 0, 'threshold must be non-negative');
  }

  const fmtIsInt = format.includes('int');
  const fmtIsNorm = format.includes('norm');
  const fmtIsFloat = format.includes('float');

  const tvc = {};
  if (fmtIsInt) {
    tvc.predicate = (coords) =>
    comparePerComponent(actTexelView.color(coords), expTexelView.color(coords), maxIntDiff);
  } else if (fmtIsNorm && maxDiffULPsForNormFormat !== undefined) {
    tvc.predicate = (coords) =>
    comparePerComponent(
      actTexelView.ulpFromZero(coords),
      expTexelView.ulpFromZero(coords),
      maxDiffULPsForNormFormat
    );
  } else if (fmtIsFloat && maxDiffULPsForFloatFormat !== undefined) {
    tvc.predicate = (coords) =>
    comparePerComponent(
      actTexelView.ulpFromZero(coords),
      expTexelView.ulpFromZero(coords),
      maxDiffULPsForFloatFormat
    );
  } else if (maxFractionalDiff !== undefined) {
    tvc.predicate = (coords) =>
    comparePerComponent(
      actTexelView.color(coords),
      expTexelView.color(coords),
      maxFractionalDiff
    );
  } else {
    if (fmtIsNorm) {
      unreachable('need maxFractionalDiff or maxDiffULPsForNormFormat to compare norm textures');
    } else if (fmtIsFloat) {
      unreachable('need maxFractionalDiff or maxDiffULPsForFloatFormat to compare float textures');
    } else {
      unreachable();
    }
  }

  const repr = kTexelRepresentationInfo[format];
  if (fmtIsInt) {
    tvc.tableRows = (failedCoords) => [
    [`tolerance ± ${maxIntDiff}`],
    function* () {
      yield* [` diff (act - exp)`, '==', ''];
      for (const coords of failedCoords) {
        const act = actTexelView.color(coords);
        const exp = expTexelView.color(coords);
        yield repr.componentOrder.map((ch) => act[ch] - exp[ch]).join(',');
      }
    }()];

  } else if (
  fmtIsNorm && maxDiffULPsForNormFormat !== undefined ||
  fmtIsFloat && maxDiffULPsForFloatFormat !== undefined)
  {
    const toleranceULPs = fmtIsNorm ? maxDiffULPsForNormFormat : maxDiffULPsForFloatFormat;
    tvc.tableRows = (failedCoords) => [
    [`tolerance ± ${toleranceULPs} normal-ULPs`],
    function* () {
      yield* [` diff (act - exp) in normal-ULPs`, '==', ''];
      for (const coords of failedCoords) {
        const act = actTexelView.ulpFromZero(coords);
        const exp = expTexelView.ulpFromZero(coords);
        yield repr.componentOrder.map((ch) => act[ch] - exp[ch]).join(',');
      }
    }()];

  } else {
    assert(maxFractionalDiff !== undefined);
    tvc.tableRows = (failedCoords) => [
    [`tolerance ± ${maxFractionalDiff}`],
    function* () {
      yield* [` diff (act - exp)`, '==', ''];
      for (const coords of failedCoords) {
        const act = actTexelView.color(coords);
        const exp = expTexelView.color(coords);
        yield repr.componentOrder.map((ch) => (act[ch] - exp[ch]).toPrecision(4)).join(',');
      }
    }()];

  }

  return tvc;
}

function comparePerComponent(
actual,
expected,
maxDiff)
{
  return Object.keys(actual).every((key) => {
    const k = key;
    const act = actual[k];
    const exp = expected[k];
    if (exp === undefined) return false;
    return numbersApproximatelyEqual(act, exp, maxDiff);
  });
}

/** Create a new mappable GPUBuffer, and copy a subrectangle of GPUTexture data into it. */
function createTextureCopyForMapRead(
t,
source,
copySize,
{ format })
{
  const { byteLength, bytesPerRow, rowsPerImage } = getTextureSubCopyLayout(format, copySize, {
    aspect: source.aspect
  });

  const buffer = t.createBufferTracked({
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ,
    size: byteLength
  });

  const cmd = t.device.createCommandEncoder();
  cmd.copyTextureToBuffer(source, { buffer, bytesPerRow, rowsPerImage }, copySize);
  t.device.queue.submit([cmd.finish()]);

  return { buffer, bytesPerRow, rowsPerImage };
}

export function findFailedPixels(
format,
subrectOrigin,
subrectSize,
{ actTexelView, expTexelView },
texelCompareOptions,
coords)
{
  const comparer = makeTexelViewComparer(
    format,
    { actTexelView, expTexelView },
    texelCompareOptions
  );

  const lowerCorner = [subrectSize.width, subrectSize.height, subrectSize.depthOrArrayLayers];
  const upperCorner = [0, 0, 0];
  const failedPixels = [];
  for (const coord of coords ?? fullSubrectCoordinates(subrectOrigin, subrectSize)) {
    const { x, y, z } = coord;
    if (!comparer.predicate(coord)) {
      failedPixels.push(coord);
      lowerCorner[0] = Math.min(lowerCorner[0], x);
      lowerCorner[1] = Math.min(lowerCorner[1], y);
      lowerCorner[2] = Math.min(lowerCorner[2], z);
      upperCorner[0] = Math.max(upperCorner[0], x);
      upperCorner[1] = Math.max(upperCorner[1], y);
      upperCorner[2] = Math.max(upperCorner[2], z);
    }
  }
  if (failedPixels.length === 0) {
    return undefined;
  }

  const info = kTextureFormatInfo[format];
  const repr = kTexelRepresentationInfo[format];
  // MAINTENANCE_TODO: Print depth-stencil formats as float+int instead of float+float.
  const printAsInteger = info.color ?
  // For color, pick the type based on the format type
  ['uint', 'sint'].includes(info.color.type) :
  // Print depth as "float", depth-stencil as "float,float", stencil as "int".
  !info.depth;
  const numericToString = numericToStringBuilder(printAsInteger);

  const componentOrderStr = repr.componentOrder.join(',') + ':';

  const printCoords = function* () {
    yield* [' coords', '==', 'X,Y,Z:'];
    for (const coords of failedPixels) yield `${coords.x},${coords.y},${coords.z}`;
  }();
  const printActualBytes = function* () {
    yield* [' act. texel bytes (little-endian)', '==', '0x:'];
    for (const coords of failedPixels) {
      yield Array.from(actTexelView.bytes(coords), (b) => b.toString(16).padStart(2, '0')).join(' ');
    }
  }();
  const printActualColors = function* () {
    yield* [' act. colors', '==', componentOrderStr];
    for (const coords of failedPixels) {
      const pixel = actTexelView.color(coords);
      yield `${repr.componentOrder.map((ch) => numericToString(pixel[ch])).join(',')}`;
    }
  }();
  const printExpectedColors = function* () {
    yield* [' exp. colors', '==', componentOrderStr];
    for (const coords of failedPixels) {
      const pixel = expTexelView.color(coords);
      yield `${repr.componentOrder.map((ch) => numericToString(pixel[ch])).join(',')}`;
    }
  }();
  const printActualULPs = function* () {
    yield* [' act. normal-ULPs-from-zero', '==', componentOrderStr];
    for (const coords of failedPixels) {
      const pixel = actTexelView.ulpFromZero(coords);
      yield `${repr.componentOrder.map((ch) => pixel[ch]).join(',')}`;
    }
  }();
  const printExpectedULPs = function* () {
    yield* [` exp. normal-ULPs-from-zero`, '==', componentOrderStr];
    for (const coords of failedPixels) {
      const pixel = expTexelView.ulpFromZero(coords);
      yield `${repr.componentOrder.map((ch) => pixel[ch]).join(',')}`;
    }
  }();

  const opts = {
    fillToWidth: 120,
    numericToString
  };
  return `\
 between ${lowerCorner} and ${upperCorner} inclusive:
${generatePrettyTable(opts, [
  printCoords,
  printActualBytes,
  printActualColors,
  printExpectedColors,
  printActualULPs,
  printExpectedULPs,
  ...comparer.tableRows(failedPixels)]
  )}`;
}

/**
 * Check the contents of a GPUTexture by reading it back (with copyTextureToBuffer+mapAsync), then
 * comparing the data with the data in `expTexelView`.
 *
 * The actual and expected texture data are both converted to the "NormalULPFromZero" format,
 * which is a signed number representing how far the number is from zero, in ULPs, skipping
 * subnormal numbers (where ULP is defined for float, normalized, and integer formats).
 */
export async function textureContentIsOKByT2B(
t,
source,
copySize_,
{ expTexelView },
texelCompareOptions,
coords)
{
  const subrectOrigin = reifyOrigin3D(source.origin ?? [0, 0, 0]);
  const subrectSize = reifyExtent3D(copySize_);
  const format = expTexelView.format;

  const { buffer, bytesPerRow, rowsPerImage } = createTextureCopyForMapRead(
    t,
    source,
    subrectSize,
    { format }
  );

  await buffer.mapAsync(GPUMapMode.READ);
  const data = new Uint8Array(buffer.getMappedRange());

  const texelViewConfig = {
    bytesPerRow,
    rowsPerImage,
    subrectOrigin,
    subrectSize
  };

  const actTexelView = TexelView.fromTextureDataByReference(format, data, texelViewConfig);

  const failedPixelsMessage = findFailedPixels(
    format,
    subrectOrigin,
    subrectSize,
    { actTexelView, expTexelView },
    texelCompareOptions,
    coords
  );

  if (failedPixelsMessage === undefined) {
    return undefined;
  }

  const msg = 'Texture level had unexpected contents:\n' + failedPixelsMessage;
  return new ErrorWithExtra(msg, () => ({
    expTexelView,
    // Make a new TexelView with a copy of the data so we can unmap the buffer (debug mode only).
    actTexelView: TexelView.fromTextureDataByReference(format, data.slice(), texelViewConfig)
  }));
}