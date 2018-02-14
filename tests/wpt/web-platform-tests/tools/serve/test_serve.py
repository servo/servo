from . import serve

def test_make_hosts_file():
    hosts = serve.make_hosts_file({
        "domains": {"www": "www.foo.bar.test", "www1": "www1.foo.bar.test"},
        "not_domains": {"aaa": "aaa.foo.bar.test", "bbb": "bbb.foo.bar.test"}
    }, "127.1.1.1")
    lines = hosts.split("\n")
    assert "127.1.1.1\twww.foo.bar.test" in lines
    assert "127.1.1.1\twww1.foo.bar.test" in lines
    assert "0.0.0.0\taaa.foo.bar.test" in lines
    assert "0.0.0.0\tbbb.foo.bar.test" in lines
