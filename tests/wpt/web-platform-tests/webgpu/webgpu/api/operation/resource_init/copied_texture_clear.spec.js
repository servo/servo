/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = 'Test uninitialized textures are initialized to zero when copied.';
import * as C from '../../../../common/constants.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../../common/framework/util/util.js';
import { ReadMethod, TextureZeroInitTest } from './texture_zero_init_test.js';

class CopiedTextureClearTest extends TextureZeroInitTest {
  checkContentsByBufferCopy(texture, state, subresourceRange) {
    for (const {
      level: mipLevel,
      slice
    } of subresourceRange.each()) {
      assert(this.params.dimension === '2d');
      this.expectSingleColor(texture, this.params.format, {
        size: [this.textureWidth, this.textureHeight, 1],
        dimension: this.params.dimension,
        slice,
        layout: {
          mipLevel
        },
        exp: this.stateToTexelComponents[state]
      });
    }
  }

  checkContentsByTextureCopy(texture, state, subresourceRange) {
    for (const {
      level,
      slice
    } of subresourceRange.each()) {
      assert(this.params.dimension === '2d');
      const width = this.textureWidth >> level;
      const height = this.textureHeight >> level;
      const dst = this.device.createTexture({
        size: [width, height, 1],
        format: this.params.format,
        usage: C.TextureUsage.CopyDst | C.TextureUsage.CopySrc
      });
      const commandEncoder = this.device.createCommandEncoder();
      commandEncoder.copyTextureToTexture({
        texture,
        mipLevel: level,
        arrayLayer: slice
      }, {
        texture: dst,
        mipLevel: 0,
        arrayLayer: 0
      }, {
        width,
        height,
        depth: 1
      });
      this.queue.submit([commandEncoder.finish()]);
      this.expectSingleColor(dst, this.params.format, {
        size: [width, height, 1],
        exp: this.stateToTexelComponents[state]
      });
    }
  }

  checkContents(texture, state, subresourceRange) {
    switch (this.params.readMethod) {
      case ReadMethod.CopyToBuffer:
        this.checkContentsByBufferCopy(texture, state, subresourceRange);
        break;

      case ReadMethod.CopyToTexture:
        this.checkContentsByTextureCopy(texture, state, subresourceRange);
        break;

      default:
        unreachable();
    }
  }

}

export const g = makeTestGroup(CopiedTextureClearTest);
g.test('uninitialized_texture_is_zero').params(TextureZeroInitTest.generateParams([ReadMethod.CopyToBuffer, ReadMethod.CopyToTexture])).fn(t => {
  t.run();
});
//# sourceMappingURL=copied_texture_clear.spec.js.map