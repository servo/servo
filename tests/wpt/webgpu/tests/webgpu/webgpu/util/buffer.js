/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { memcpy } from '../../common/util/util.js';import { align } from './math.js';

/**
 * Creates a buffer with the contents of some TypedArray.
 * The buffer size will always be aligned to 4 as we set mappedAtCreation === true when creating the
 * buffer.
 */
export function makeBufferWithContents(
device,
dataArray,
usage)
{
  const buffer = device.createBuffer({
    mappedAtCreation: true,
    size: align(dataArray.byteLength, 4),
    usage
  });
  memcpy({ src: dataArray }, { dst: buffer.getMappedRange() });
  buffer.unmap();
  return buffer;
}