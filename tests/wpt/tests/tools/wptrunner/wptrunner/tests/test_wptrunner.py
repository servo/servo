from ..wptrunner import get_pause_after_test
from .test_testloader import Subsuite, TestFilter, TestLoader, WPTManifest

def test_get_pause_after_test():  # type: ignore
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
        "version": 8,
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
