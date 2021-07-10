/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ import { Fixture } from '../../common/framework/fixture.js';
import { assert } from '../../common/framework/util/util.js';

export class IDLTest extends Fixture {
  // TODO: add a helper to check prototype chains

  /**
   * Asserts that a member of an IDL interface has the expected value.
   */
  assertMember(act, exp, key) {
    assert(key in act, () => `Expected key ${key} missing`);
    assert(act[key] === exp[key], () => `Value of [${key}] was ${act[key]}, expected ${exp[key]}`);
  }

  /**
   * Asserts that an IDL interface has the same number of keys as the
   *
   * TODO: add a way to check for the types of keys with unknown values, like methods and attributes
   * TODO: handle extensions
   */
  assertMemberCount(act, exp) {
    const expKeys = Object.keys(exp);
    const actKeys = Object.keys(act);
    assert(
      actKeys.length === expKeys.length,
      () => `Had ${actKeys.length} keys, expected ${expKeys.length}`
    );
  }
}
