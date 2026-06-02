# mypy: allow-untyped-calls

from pathlib import Path
from typing import List, Optional, Tuple
from unittest.mock import Mock

from ..testloader import TestQueueBuilder
from ..wptcommandline import TestRoot
from ..wptrunner import get_loader, get_pause_after_test
from .test_testloader import Subsuite, TestFilter, TestLoader, WPTManifest

TestQueueBuilder.__test__ = None  # type: ignore[attr-defined]
TestRoot.__test__ = None  # type: ignore[attr-defined]


def test_get_pause_after_test() -> None:
    manifest_json = {
        "items": {
            "testharness": {
                "a": {
                    "foo.html": [
                        "abcdef123456",
                        [None, {}],
                    ],
                    "bar.h2.html": [
                        "uvwxyz987654",
                        [None, {}],
                    ],
                }
            },
            "reftest": {
                "a": {
                    "reffoo.html": [
                        "abcdef654321",
                        [None, [["/common/some-ref.html", "=="]], {}]
                    ],
                }
            }
        },
        "url_base": "/",
        "version": 9,
    }

    kwargs = {
        "pause_after_test": None,
        "repeat_until_unexpected": False,
        "headless": False,
        "debug_test": False,
        "repeat": 1,
        "rerun": 1
    }

    manifest = WPTManifest.from_json("/", manifest_json)
    test_manifests = {manifest: {"metadata_path": ""}}

    manifest_filters = [TestFilter(test_manifests, include=["/a/foo.html", "/a/reffoo.html"])]

    subsuites = {}
    subsuites[""] = Subsuite("", config={})

    # This has two testharness tests, so shouldn't set pause_after_test
    loader = TestLoader(test_manifests, ["testharness"], None, subsuites)

    assert get_pause_after_test(loader, **kwargs) is False

    # This has one testharness test, so should set pause_after_test
    loader = TestLoader(test_manifests, ["testharness"], None, subsuites,
                        manifest_filters=manifest_filters)

    assert get_pause_after_test(loader, **kwargs) is True

    # This has one testharness test, and one reftest so shouldn't set pause_after_test
    loader = TestLoader(test_manifests, ["testharness", "reftest"], None, subsuites,
                        manifest_filters=manifest_filters)

    assert get_pause_after_test(loader, **kwargs) is False

    # This has one reftest so shouldn't set pause_after_test
    loader = TestLoader(test_manifests, ["reftest"], None, subsuites)

    assert get_pause_after_test(loader, **kwargs) is False

    multi_subsuites = {}
    multi_subsuites[""] = Subsuite("", config={})
    multi_subsuites["extra"] = Subsuite("extra", config={}, include=["/a/foo.html"])

    # This has one testharness test per subsuite, so shouldn't set pause_after_test
    loader = TestLoader(test_manifests, ["testharness"], None, multi_subsuites,
                        manifest_filters=manifest_filters)
    print(loader.tests)
    assert get_pause_after_test(loader, **kwargs) is False


def get_loader_with_fakes(
    tmp_path: Path,
    include: Optional[List[str]] = None,
    include_file: Optional[str] = None,
    exclude: Optional[List[str]] = None,
    exclude_file: Optional[str] = None,
) -> Tuple[TestQueueBuilder, TestLoader]:
    repo_root = tmp_path / "wpt"
    repo_root.mkdir()

    test_paths = {
        "/": TestRoot(str(repo_root), str(repo_root), str(repo_root / "MANIFEST.json"))
    }

    spec_dir = repo_root / "fake-spec"
    spec_dir.mkdir()
    test_filenames = [f"test-{i:03d}.html" for i in range(10)]
    for test_filename in test_filenames:
        with (spec_dir / test_filename).open("wb") as f:
            f.write(b"<script src=/resources/testharness.js></script>")

    product = Mock(spec=["name", "run_info_extras"], name="fake-product")
    product.run_info_extras = Mock(spec=[], return_value={})

    # Unfortunately this requires quite a lot of kwargs
    return get_loader(
        test_paths,
        product,
        chunk_type="none",
        debug=None,
        default_exclude=False,
        enable_webtransport_h3=True,
        exclude=exclude,
        exclude_file=exclude_file,
        exclude_tags=None,
        fully_parallel=False,
        include=include,
        include_file=include_file,
        include_manifest=None,
        manifest_download=False,
        manifest_update=True,
        processes=1,
        run_by_dir=False,
        run_info=str(repo_root / "tools/wptrunner/wptrunner.default.ini"),
        skip_crash=False,
        skip_implementation_status=None,
        skip_timeout=False,
        ssl_type="pregenerated",
        subsuite_file=None,
        subsuites=None,
        tags=None,
        test_groups_file=None,
        test_types={"crashtest", "print-reftest", "reftest", "testharness", "wdspec"},
        this_chunk=1,
        total_chunks=1,
    )


def test_get_loader(tmp_path: Path) -> None:
    _, test_loader = get_loader_with_fakes(tmp_path)

    assert test_loader.test_ids == [
        "/fake-spec/test-000.html",
        "/fake-spec/test-001.html",
        "/fake-spec/test-002.html",
        "/fake-spec/test-003.html",
        "/fake-spec/test-004.html",
        "/fake-spec/test-005.html",
        "/fake-spec/test-006.html",
        "/fake-spec/test-007.html",
        "/fake-spec/test-008.html",
        "/fake-spec/test-009.html",
    ]


def test_get_loader_include(tmp_path: Path) -> None:
    _, test_loader = get_loader_with_fakes(
        tmp_path,
        include=["/fake-spec/test-007.html", "/fake-spec/test-008.html"],
    )

    assert test_loader.test_ids == [
        "/fake-spec/test-007.html",
        "/fake-spec/test-008.html",
    ]


def test_get_loader_exclude(tmp_path: Path) -> None:
    _, test_loader = get_loader_with_fakes(
        tmp_path,
        exclude=["/fake-spec/test-007.html"],
    )

    assert test_loader.test_ids == [
        "/fake-spec/test-000.html",
        "/fake-spec/test-001.html",
        "/fake-spec/test-002.html",
        "/fake-spec/test-003.html",
        "/fake-spec/test-004.html",
        "/fake-spec/test-005.html",
        "/fake-spec/test-006.html",
        "/fake-spec/test-008.html",
        "/fake-spec/test-009.html",
    ]


def test_get_loader_include_exclude(tmp_path: Path) -> None:
    _, test_loader = get_loader_with_fakes(
        tmp_path,
        include=["/fake-spec/test-007.html", "/fake-spec/test-008.html"],
        exclude=["/fake-spec/test-007.html"],
    )

    assert test_loader.test_ids == [
        "/fake-spec/test-008.html",
    ]


def test_get_loader_include_file(tmp_path: Path) -> None:
    include = ["/fake-spec/test-007.html", "/fake-spec/test-008.html"]

    with (tmp_path / "include.txt").open("w") as f:
        f.writelines([f"{t}\n" for t in include])

    _, test_loader = get_loader_with_fakes(
        tmp_path,
        include_file=str(tmp_path / "include.txt"),
    )

    assert test_loader.test_ids == [
        "/fake-spec/test-007.html",
        "/fake-spec/test-008.html",
    ]


def test_get_loader_exclude_file(tmp_path: Path) -> None:
    exclude = ["/fake-spec/test-007.html"]

    with (tmp_path / "exclude.txt").open("w") as f:
        f.writelines([f"{t}\n" for t in exclude])

    _, test_loader = get_loader_with_fakes(
        tmp_path,
        exclude_file=str(tmp_path / "exclude.txt"),
    )

    assert test_loader.test_ids == [
        "/fake-spec/test-000.html",
        "/fake-spec/test-001.html",
        "/fake-spec/test-002.html",
        "/fake-spec/test-003.html",
        "/fake-spec/test-004.html",
        "/fake-spec/test-005.html",
        "/fake-spec/test-006.html",
        "/fake-spec/test-008.html",
        "/fake-spec/test-009.html",
    ]


def test_get_loader_include_exclude_file(tmp_path: Path) -> None:
    include = ["/fake-spec/test-007.html", "/fake-spec/test-008.html"]
    exclude = ["/fake-spec/test-007.html"]

    with (tmp_path / "include.txt").open("w") as f:
        f.writelines([f"{t}\n" for t in include])

    with (tmp_path / "exclude.txt").open("w") as f:
        f.writelines([f"{t}\n" for t in exclude])

    _, test_loader = get_loader_with_fakes(
        tmp_path,
        include_file=str(tmp_path / "include.txt"),
        exclude_file=str(tmp_path / "exclude.txt"),
    )

    assert test_loader.test_ids == [
        "/fake-spec/test-008.html",
    ]
