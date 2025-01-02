# mypy: allow-untyped-defs

import io
import textwrap

from .. import metadata
from .. import manifestupdate
from .. import wptmanifest


def test_unconditional_default_promotion():
    contents_before = io.BytesIO(
        textwrap.dedent(
            """\
            [b.html]
            """).encode())
    manifest = manifestupdate.compile(
        contents_before,
        test_path='a/b.html',
        url_base='/',
        run_info_properties=(['os'], {'os': ['version']}),
        update_intermittent=True,
        remove_intermittent=False)
    test = manifest.get_test('/a/b.html')
    test.set_result(
        metadata.RunInfo({'os': 'linux', 'version': 'jammy'}),
        metadata.Result('TIMEOUT', [], 'PASS'))
    test.set_result(
        metadata.RunInfo({'os': 'win', 'version': '10'}),
        metadata.Result('TIMEOUT', [], 'PASS'))
    test.set_result(
        metadata.RunInfo({'os': 'mac', 'version': '11'}),
        metadata.Result('FAIL', [], 'PASS'))
    test.set_result(
        metadata.RunInfo({'os': 'mac', 'version': '12'}),
        metadata.Result('FAIL', [], 'PASS'))
    test.set_result(
        metadata.RunInfo({'os': 'mac', 'version': '13'}),
        metadata.Result('FAIL', [], 'PASS'))
    test.update(full_update=True, disable_intermittent=False)

    # The conditions before the default is created will look like:
    #   expected:
    #     if os == "linux": TIMEOUT
    #     if os == "win": TIMEOUT
    #     if os == "mac": FAIL
    #
    # The update should prefer promoting `TIMEOUT` over `FAIL`, since the former
    # eliminates more conditions (both non-mac ones).
    contents_after = io.BytesIO(
        textwrap.dedent(
            """\
            [b.html]
              expected:
                if os == "mac": FAIL
                TIMEOUT
            """).encode())
    assert manifest.node == wptmanifest.parse(contents_after)
