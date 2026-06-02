# mypy: allow-untyped-defs

import io

from tools.ci import check_for_updated_refs


PUSH_OUTPUT = """To github.com:web-platform-tests/wpt.git
=	refs/heads/stable:refs/heads/stable	[up to date]
-	:refs/heads/deleted	[deleted]
+	refs/heads/force:refs/heads/force	7151480...62ee8b6 (forced update)
*	refs/heads/new:refs/heads/new	[new branch]
*	refs/tags/v1.0:refs/tags/v1.0	[new tag]
*	refs/review/pr/123:refs/review/pr/123	[new reference]
\x20	refs/heads/main:refs/heads/main	2e0d156..fd86363
\x20	HEAD:refs/heads/feature	49d4a17..5105902
\x20	refs/heads/short-sha:refs/heads/short-sha	9731..e320
\x20	refs/heads/long-sha:refs/heads/long-sha	01e1bec7812eb37679c0b1cb436290a83cbc4694..8ec0c04d22002009e9db16f5e3c5518574907c93
!	refs/heads/reject1:refs/heads/reject1	[rejected] (non-fast-forward)
!	refs/heads/reject2:refs/heads/reject2	[rejected] (already exists)
!	refs/heads/reject3:refs/heads/reject3	[rejected] (fetch first)
!	refs/heads/reject4:refs/heads/reject4	[rejected] (needs force)
!	refs/heads/reject5:refs/heads/reject5	[rejected] (stale info)
!	refs/heads/reject6:refs/heads/reject6	[rejected] (remote ref updated since checkout)
!	refs/heads/reject7:refs/heads/reject7	[rejected] (new shallow roots not allowed)
!	refs/heads/reject8:refs/heads/reject8	[rejected] (atomic push failed)
!	refs/heads/reject9:refs/heads/reject9	[remote rejected] (hook declined)
!	refs/heads/reject10:refs/heads/reject10	[remote failure] (timeout)
!	refs/heads/reject11:refs/heads/reject11	[no match]
Done
"""


def test_parse_push():
    s = io.StringIO(PUSH_OUTPUT)

    actual = list(check_for_updated_refs.parse_push(s))
    expected = [
        {
            "flag": "=",
            "from_ref": "refs/heads/stable",
            "to_ref": "refs/heads/stable",
            "summary": "[up to date]",
            "reason": None,
        },
        {
            "flag": "-",
            "from_ref": "",
            "to_ref": "refs/heads/deleted",
            "summary": "[deleted]",
            "reason": None,
        },
        {
            "flag": "+",
            "from_ref": "refs/heads/force",
            "to_ref": "refs/heads/force",
            "summary": "7151480...62ee8b6",
            "reason": "forced update",
        },
        {
            "flag": "*",
            "from_ref": "refs/heads/new",
            "to_ref": "refs/heads/new",
            "summary": "[new branch]",
            "reason": None,
        },
        {
            "flag": "*",
            "from_ref": "refs/tags/v1.0",
            "to_ref": "refs/tags/v1.0",
            "summary": "[new tag]",
            "reason": None,
        },
        {
            "flag": "*",
            "from_ref": "refs/review/pr/123",
            "to_ref": "refs/review/pr/123",
            "summary": "[new reference]",
            "reason": None,
        },
        {
            "flag": " ",
            "from_ref": "refs/heads/main",
            "to_ref": "refs/heads/main",
            "summary": "2e0d156..fd86363",
            "reason": None,
        },
        {
            "flag": " ",
            "from_ref": "HEAD",
            "to_ref": "refs/heads/feature",
            "summary": "49d4a17..5105902",
            "reason": None,
        },
        {
            "flag": " ",
            "from_ref": "refs/heads/short-sha",
            "to_ref": "refs/heads/short-sha",
            "summary": "9731..e320",
            "reason": None,
        },
        {
            "flag": " ",
            "from_ref": "refs/heads/long-sha",
            "to_ref": "refs/heads/long-sha",
            "summary": "01e1bec7812eb37679c0b1cb436290a83cbc4694..8ec0c04d22002009e9db16f5e3c5518574907c93",
            "reason": None,
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject1",
            "to_ref": "refs/heads/reject1",
            "summary": "[rejected]",
            "reason": "non-fast-forward",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject2",
            "to_ref": "refs/heads/reject2",
            "summary": "[rejected]",
            "reason": "already exists",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject3",
            "to_ref": "refs/heads/reject3",
            "summary": "[rejected]",
            "reason": "fetch first",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject4",
            "to_ref": "refs/heads/reject4",
            "summary": "[rejected]",
            "reason": "needs force",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject5",
            "to_ref": "refs/heads/reject5",
            "summary": "[rejected]",
            "reason": "stale info",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject6",
            "to_ref": "refs/heads/reject6",
            "summary": "[rejected]",
            "reason": "remote ref updated since checkout",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject7",
            "to_ref": "refs/heads/reject7",
            "summary": "[rejected]",
            "reason": "new shallow roots not allowed",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject8",
            "to_ref": "refs/heads/reject8",
            "summary": "[rejected]",
            "reason": "atomic push failed",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject9",
            "to_ref": "refs/heads/reject9",
            "summary": "[remote rejected]",
            "reason": "hook declined",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject10",
            "to_ref": "refs/heads/reject10",
            "summary": "[remote failure]",
            "reason": "timeout",
        },
        {
            "flag": "!",
            "from_ref": "refs/heads/reject11",
            "to_ref": "refs/heads/reject11",
            "summary": "[no match]",
            "reason": None,
        },
    ]

    assert expected == actual


def test_parse_push_malformed_lines():
    s = io.StringIO(
        """To github.com:test/repo.git
\x20	refs/heads/good:refs/heads/good	be30154..bd80959
malformed line without tabs
another bad line
\x20	refs/heads/good2:refs/heads/good2	ae0f96f..207e07b
Done
"""
    )

    actual = list(check_for_updated_refs.parse_push(s))
    expected = [
        {
            "flag": " ",
            "from_ref": "refs/heads/good",
            "to_ref": "refs/heads/good",
            "summary": "be30154..bd80959",
            "reason": None,
        },
        {
            "flag": " ",
            "from_ref": "refs/heads/good2",
            "to_ref": "refs/heads/good2",
            "summary": "ae0f96f..207e07b",
            "reason": None,
        },
    ]

    assert expected == actual


def test_process_push():
    s = io.StringIO(PUSH_OUTPUT)

    actual = check_for_updated_refs.process_push(
        s,
        [
            "refs/heads/stable",  # Omitted as it wasn't actually updated.
            "refs/heads/deleted",
            "refs/heads/force",
            "refs/heads/new",
            "refs/tags/v1.0",
            "refs/review/pr/123",
            "refs/heads/main",
            "refs/heads/feature",
            "refs/heads/reject1",  # Omitted as it wasn't actually updated.
            "refs/heads/nonexistent",  # Omitted as it wasn't actually updated.
        ],
    )
    expected = {
        "refs/heads/deleted": None,
        "refs/heads/force": "62ee8b6",
        "refs/heads/new": None,
        "refs/tags/v1.0": None,
        "refs/review/pr/123": None,
        "refs/heads/main": "fd86363",
        "refs/heads/feature": "5105902",
    }

    assert expected == actual
