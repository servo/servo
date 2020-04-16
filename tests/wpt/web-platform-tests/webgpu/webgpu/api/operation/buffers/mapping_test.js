/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { GPUTest } from '../../../gpu_test.js';
export class MappingTest extends GPUTest {
  checkMapWrite(buffer, mappedContents, size) {
    this.checkMapWriteZeroed(mappedContents, size);
    const mappedView = new Uint32Array(mappedContents);
    const expected = new Uint32Array(new ArrayBuffer(size));
    this.expect(mappedView.byteLength === size);

    for (let i = 0; i < mappedView.length; ++i) {
      mappedView[i] = expected[i] = i + 1;
    }

    buffer.unmap();
    this.expectContents(buffer, expected);
  }

  checkMapWriteZeroed(arrayBuffer, expectedSize) {
    this.expect(arrayBuffer.byteLength === expectedSize);
    const view = new Uint8Array(arrayBuffer);
    this.expectZero(view);
  }

  expectZero(actual) {
    const size = actual.byteLength;

    for (let i = 0; i < size; ++i) {
      if (actual[i] !== 0) {
        this.fail(`at [${i}], expected zero, got ${actual[i]}`);
        break;
      }
    }
  }

}
//# sourceMappingURL=mapping_test.js.map