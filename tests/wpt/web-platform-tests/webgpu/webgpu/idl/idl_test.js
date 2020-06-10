/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { Fixture } from '../../common/framework/fixture.js';
import { assert } from '../../common/framework/util/util.js';
export class IDLTest extends Fixture {
  /**
   * Asserts that an IDL interface has the expected members.
   */
  // TODO: exp should allow sentinel markers for unnameable values, such as methods and attributes
  // TODO: handle extensions
  // TODO: check prototype chains (maybe as separate method)
  assertMembers(act, exp) {
    const expKeys = Object.keys(exp);

    for (const k of expKeys) {
      assert(k in act, () => `Expected key ${k} missing`);
      assert(act[k] === exp[k], () => `Value of [${k}] was ${act[k]}, expected ${exp[k]}`);
    }

    const actKeys = Object.keys(act);
    assert(actKeys.length === expKeys.length, () => `Had ${actKeys.length} keys, expected ${expKeys.length}`);
  }

}
//# sourceMappingURL=idl_test.js.map