# mypy: allow-untyped-defs

import io

from tools.ci import check_for_updated_refs


def test_parse_push():
    s = io.StringIO(
        """
To github.com:gsnedders/web-platform-tests.git
=	refs/heads/a:refs/heads/a	[up to date]
-	:refs/heads/b	[deleted]
+	refs/heads/c:refs/heads/c	a6eb923e19...9b6507e295 (forced update)
*	refs/heads/d:refs/heads/d	[new branch]
\x20	refs/heads/e:refs/heads/e	0acd8f62f1..6188942729
!	refs/heads/f:refs/heads/f	[rejected] (atomic push failed)
Done
    """
    )

    actual = list(check_for_updated_refs.parse_push(s))
    print(repr(actual))
    expected = [
        {
            "flag": "=",
            "from": "refs/heads/a",
            "to": "refs/heads/a",
            "summary": "[up to date]",
            "reason": None,
        },
        {
            "flag": "-",
            "from": "",
            "to": "refs/heads/b",
            "summary": "[deleted]",
            "reason": None,
        },
        {
            "flag": "+",
            "from": "refs/heads/c",
            "to": "refs/heads/c",
            "summary": "a6eb923e19...9b6507e295",
            "reason": "forced update",
        },
        {
            "flag": "*",
            "from": "refs/heads/d",
            "to": "refs/heads/d",
            "summary": "[new branch]",
            "reason": None,
        },
        {
            "flag": " ",
            "from": "refs/heads/e",
            "to": "refs/heads/e",
            "summary": "0acd8f62f1..6188942729",
            "reason": None,
        },
        {
            "flag": "!",
            "from": "refs/heads/f",
            "to": "refs/heads/f",
            "summary": "[rejected]",
            "reason": "atomic push failed",
        },
    ]

    assert expected == actual


def test_process_push():
    s = io.StringIO(
        """
To github.com:gsnedders/web-platform-tests.git
=	refs/heads/a:refs/heads/a	[up to date]
-	:refs/heads/b	[deleted]
+	refs/heads/c:refs/heads/c	a6eb923e19...9b6507e295 (forced update)
*	refs/heads/d:refs/heads/d	[new branch]
\x20	refs/heads/e:refs/heads/e	0acd8f62f1..6188942729
!	refs/heads/f:refs/heads/f	[rejected] (atomic push failed)
Done
    """
    )

    actual = list(
        check_for_updated_refs.process_push(
            s,
            [
                "refs/heads/e",
                "refs/heads/b",
                "refs/heads/c",
                "refs/heads/d",
                "refs/heads/e",
                "refs/heads/x",
            ],
        )
    )
    expected = [
        "refs/heads/b",
        "refs/heads/c",
        "refs/heads/d",
        "refs/heads/e",
    ]

    assert expected == actual
