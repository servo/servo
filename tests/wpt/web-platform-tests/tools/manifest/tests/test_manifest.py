from .. import manifest, item as manifestitem, sourcefile


def test_local_reftest_add():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.local_changes.add(test)
    assert list(m) == [(test.path, {test})]


def test_local_reftest_delete_path():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.add(test)
    m.local_changes.add_deleted(test.path)
    assert list(m) == []


def test_local_reftest_adjusted():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.add(test)

    assert list(m) == [(test.path, {test})]

    assert m.compute_reftests({test.path: {test}}) == {test}

    test_1 = manifestitem.RefTest(s, "/test-1", [("/test", "==")])
    m.local_changes.add(test_1)

    assert m.compute_reftests({test.path: {test}, test_1.path: {test_1}}) == {test_1}

    m.local_changes._deleted_reftests[test.path] = {test}

    assert list(m) == [(test_1.path, {test_1})]


def test_manifest_to_json():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.add(test)
    test_1 = manifestitem.RefTest(s, "/test-1", [("/test", "==")])
    m.local_changes.add(test_1)
    m.local_changes._deleted_reftests[test.path] = {test}

    json_str = m.to_json()
    loaded = manifest.Manifest.from_json("/", json_str)

    assert list(loaded) == list(m)

    assert loaded.to_json() == json_str
