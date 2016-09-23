from .. import manifest, item as manifestitem, sourcefile


def test_local_reftest_add():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.local_changes.add(test)
    m.update_reftests()
    assert list(m) == [(test.path, {test})]


def test_local_reftest_delete_path():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.add(test)
    m.local_changes.add_deleted(test.path)
    m.update_reftests()
    assert list(m) == []


def test_local_reftest_adjusted():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.add(test)
    m.update_reftests()

    assert m.compute_reftests({test.path: {test}}) == {test}

    assert list(m) == [(test.path, {test})]

    s_1 = sourcefile.SourceFile("/", "test-1", "/")
    test_1 = manifestitem.RefTest(s_1, "/test-1", [("/test", "==")])
    m.local_changes.add(test_1)
    m.update_reftests()

    assert m.compute_reftests({test.path: {test}, test_1.path: {test_1}}) == {test_1}

    assert list(m) == [(test_1.path, {test_1})]


def test_manifest_to_json():
    m = manifest.Manifest()
    s = sourcefile.SourceFile("/", "test", "/")
    test = manifestitem.RefTest(s, "/test", [("/ref", "==")])
    m.add(test)
    s_1 = sourcefile.SourceFile("/", "test-1", "/")
    test_1 = manifestitem.RefTest(s_1, "/test-1", [("/test", "==")])
    m.local_changes.add(test_1)
    m.local_changes.add_deleted(test.path)
    m.update_reftests()

    json_str = m.to_json()
    loaded = manifest.Manifest.from_json("/", json_str)

    assert list(loaded) == list(m)

    assert loaded.to_json() == json_str


def test_reftest_computation_chain():
    m = manifest.Manifest()

    s1 = sourcefile.SourceFile("/", "test1", "/")
    s2 = sourcefile.SourceFile("/", "test2", "/")

    test1 = manifestitem.RefTest(s1, "/test1", [("/test3", "==")])
    test2 = manifestitem.RefTest(s2, "/test2", [("/test1", "==")])
    m.add(test1)
    m.add(test2)

    m.update_reftests()

    assert m.reftest_nodes == {'test1': {test1},
                               'test2': {test2}}

    assert list(m) == [("test2", {test2})]
    assert list(m.local_changes.itertypes()) == []
