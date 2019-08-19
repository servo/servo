/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = ``;
import { TestGroup } from '../../../framework/index.js';
import { GPUTest } from '../gpu_test.js'; // TODO: doesn't need to use GPUTest

export const g = new TestGroup(GPUTest);
g.test('getContext returns GPUCanvasContext', async t => {
  const canvas = document.createElement('canvas');
  canvas.width = 10;
  canvas.height = 10; // TODO: fix types so these aren't necessary

  const ctx = canvas.getContext('gpupresent');
  t.expect(ctx instanceof window.GPUCanvasContext);
});
//# sourceMappingURL=context_creation.spec.js.map