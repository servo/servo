/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = ``;
import { Fixture } from '../../../common/framework/fixture.js';
import { TestGroup } from '../../../common/framework/test_group.js';
export const g = new TestGroup(Fixture);
g.test('canvas element getContext returns GPUCanvasContext', async t => {
  if (typeof document === 'undefined') {
    // Skip if there is no document (Workers, Node)
    t.skip('DOM is not available to create canvas element');
  }

  const canvas = document.createElement('canvas');
  canvas.width = 10;
  canvas.height = 10; // TODO: fix types so these aren't necessary

  const ctx = canvas.getContext('gpupresent');
  t.expect(ctx instanceof window.GPUCanvasContext);
});
//# sourceMappingURL=context_creation.spec.js.map