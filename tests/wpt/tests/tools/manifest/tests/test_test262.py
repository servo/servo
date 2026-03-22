# mypy: allow-untyped-defs

import pytest

from tools.manifest.log import get_logger
from tools.manifest.test262 import parse, TestRecord

TestRecord.__test__ = False  # type: ignore[attr-defined]

@pytest.mark.parametrize("name, src, expected_record", [
    (
        "test.js",
        """/*---
description: A simple test
features: [Test262]
---*/
assert.sameValue(1, 1);
""",
        TestRecord("""/*---
description: A simple test
features: [Test262]
---*/
assert.sameValue(1, 1);
""", includes=None, negative=None, is_module=False, is_only_strict=False)
    ),
    (
        "no_frontmatter.js",
        """assert.sameValue(1, 1);""",
        None
    ),
    (
        "test_FIXTURE.js",
        """/*---
description: A fixture file
---*/
assert.sameValue(1, 1);
""",
        None
    ),
    (
        "flags-module.js",
        """/*---
description: Test with module flag
flags: [raw, module]
---*/
assert.sameValue(1, 1);
""",
        TestRecord("""/*---
description: Test with module flag
flags: [raw, module]
---*/
assert.sameValue(1, 1);
""", includes=None, negative=None, is_module=True, is_only_strict=False)
    ),
    (
        "flags-onlyStrict.js",
        """/*---
description: Test with onlyStrict flag
flags: [raw, onlyStrict]
---*/
assert.sameValue(1, 1);
""",
        TestRecord("""/*---
description: Test with onlyStrict flag
flags: [raw, onlyStrict]
---*/
assert.sameValue(1, 1);
""", includes=None, negative=None, is_module=False, is_only_strict=True)
    ),
    (
        "negative.js",
        """/*---
description: Negative test
negative:
  phase: runtime
  type: TypeError
---*/
throw new TypeError();
""",
        TestRecord("""/*---
description: Negative test
negative:
  phase: runtime
  type: TypeError
---*/
throw new TypeError();
""", includes=None, negative={"phase": "runtime", "type": "TypeError"}, is_module=False, is_only_strict=False)
    ),
    (
        "includes.js",
        """/*---
description: Test with includes
includes: [assert.js, sta.js]
---*/
assert.sameValue(1, 1);
""",
        TestRecord("""/*---
description: Test with includes
includes: [assert.js, sta.js]
---*/
assert.sameValue(1, 1);
""", includes=["assert.js", "sta.js"], negative=None, is_module=False, is_only_strict=False)
    ),
])
def test_test262_parser(name, src, expected_record):
    record = parse(get_logger(), src, name)

    assert expected_record == record
