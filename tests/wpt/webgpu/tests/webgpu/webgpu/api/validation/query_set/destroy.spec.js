/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Destroying a query set more than once is allowed.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('twice').fn(t => {
  const qset = t.device.createQuerySet({ type: 'occlusion', count: 1 });

  qset.destroy();
  qset.destroy();
});

g.test('invalid_queryset')
  .desc('Test that invalid querysets may be destroyed without generating validation errors.')
  .fn(async t => {
    t.device.pushErrorScope('validation');

    const invalidQuerySet = t.device.createQuerySet({
      type: 'occlusion',
      count: 4097, // 4096 is the limit
    });

    // Expect error because it's invalid.
    const error = await t.device.popErrorScope();
    t.expect(!!error);

    // This line should not generate an error
    invalidQuerySet.destroy();
  });
