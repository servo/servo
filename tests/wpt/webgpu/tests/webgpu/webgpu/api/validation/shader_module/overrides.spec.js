/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
This tests overrides numeric identifiers should not conflict.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('id_conflict')
  .desc(
    `
Tests that overrides' explicit numeric identifier should not conflict.
`
  )
  .fn(t => {
    t.expectValidationError(() => {
      t.device.createShaderModule({
        code: `
@id(1234) override c0: u32;
@id(4321) override c1: u32;

@compute @workgroup_size(1) fn main() {
  // make sure the overridable constants are not optimized out
  _ = c0;
  _ = c1;
}
          `,
      });
    }, false);

    t.expectValidationError(() => {
      t.device.createShaderModule({
        code: `
@id(1234) override c0: u32;
@id(1234) override c1: u32;

@compute @workgroup_size(1) fn main() {
  // make sure the overridable constants are not optimized out
  _ = c0;
  _ = c1;
}
          `,
      });
    }, true);
  });

g.test('name_conflict')
  .desc(
    `
Tests that overrides' variable name should not conflict, regardless of their numeric identifiers.
`
  )
  .fn(t => {
    t.expectValidationError(() => {
      t.device.createShaderModule({
        code: `
override c0: u32;
override c0: u32;

@compute @workgroup_size(1) fn main() {
  // make sure the overridable constants are not optimized out
  _ = c0;
}
          `,
      });
    }, true);

    t.expectValidationError(() => {
      t.device.createShaderModule({
        code: `
@id(1) override c0: u32;
override c0: u32;

@compute @workgroup_size(1) fn main() {
  // make sure the overridable constants are not optimized out
  _ = c0;
}
          `,
      });
    }, true);

    t.expectValidationError(() => {
      t.device.createShaderModule({
        code: `
@id(1) override c0: u32;
@id(2) override c0: u32;

@compute @workgroup_size(1) fn main() {
  // make sure the overridable constants are not optimized out
  _ = c0;
}
          `,
      });
    }, true);
  });
