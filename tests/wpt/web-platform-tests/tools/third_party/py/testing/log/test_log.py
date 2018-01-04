import py

from py._log.log import default_keywordmapper

callcapture = py.io.StdCapture.call


def setup_module(mod):
    mod._oldstate = default_keywordmapper.getstate()

def teardown_module(mod):
    default_keywordmapper.setstate(mod._oldstate)

class TestLogProducer:
    def setup_method(self, meth):
        from py._log.log import default_keywordmapper
        default_keywordmapper.setstate(_oldstate)

    def test_getstate_setstate(self):
        state = py.log._getstate()
        py.log.setconsumer("hello", [].append)
        state2 = py.log._getstate()
        assert state2 != state
        py.log._setstate(state)
        state3 = py.log._getstate()
        assert state3 == state

    def test_producer_repr(self):
        d = py.log.Producer("default")
        assert repr(d).find('default') != -1

    def test_produce_one_keyword(self):
        l = []
        py.log.setconsumer('s1', l.append)
        py.log.Producer('s1')("hello world")
        assert len(l) == 1
        msg = l[0]
        assert msg.content().startswith('hello world')
        assert msg.prefix() == '[s1] '
        assert str(msg) == "[s1] hello world"

    def test_producer_class(self):
        p = py.log.Producer('x1')
        l = []
        py.log.setconsumer(p._keywords, l.append)
        p("hello")
        assert len(l) == 1
        assert len(l[0].keywords) == 1
        assert 'x1' == l[0].keywords[0]

    def test_producer_caching(self):
        p = py.log.Producer('x1')
        x2 = p.x2
        assert x2 is p.x2

class TestLogConsumer:
    def setup_method(self, meth):
        default_keywordmapper.setstate(_oldstate)
    def test_log_none(self):
        log = py.log.Producer("XXX")
        l = []
        py.log.setconsumer('XXX', l.append)
        log("1")
        assert l
        l[:] = []
        py.log.setconsumer('XXX', None)
        log("2")
        assert not l

    def test_log_default_stderr(self):
        res, out, err = callcapture(py.log.Producer("default"), "hello")
        assert err.strip() == "[default] hello"

    def test_simple_consumer_match(self):
        l = []
        py.log.setconsumer("x1", l.append)
        p = py.log.Producer("x1 x2")
        p("hello")
        assert l
        assert l[0].content() == "hello"

    def test_simple_consumer_match_2(self):
        l = []
        p = py.log.Producer("x1 x2")
        py.log.setconsumer(p._keywords, l.append)
        p("42")
        assert l
        assert l[0].content() == "42"

    def test_no_auto_producer(self):
        p = py.log.Producer('x')
        py.test.raises(AttributeError, "p._x")
        py.test.raises(AttributeError, "p.x_y")

    def test_setconsumer_with_producer(self):
        l = []
        p = py.log.Producer("hello")
        py.log.setconsumer(p, l.append)
        p("world")
        assert str(l[0]) == "[hello] world"

    def test_multi_consumer(self):
        l = []
        py.log.setconsumer("x1", l.append)
        py.log.setconsumer("x1 x2", None)
        p = py.log.Producer("x1 x2")
        p("hello")
        assert not l
        py.log.Producer("x1")("hello")
        assert l
        assert l[0].content() == "hello"

    def test_log_stderr(self):
        py.log.setconsumer("xyz", py.log.STDOUT)
        res, out, err = callcapture(py.log.Producer("xyz"), "hello")
        assert not err
        assert out.strip() == '[xyz] hello'

    def test_log_file(self, tmpdir):
        customlog = tmpdir.join('log.out')
        py.log.setconsumer("default", open(str(customlog), 'w', 1))
        py.log.Producer("default")("hello world #1")
        assert customlog.readlines() == ['[default] hello world #1\n']

        py.log.setconsumer("default", py.log.Path(customlog, buffering=False))
        py.log.Producer("default")("hello world #2")
        res = customlog.readlines()
        assert res == ['[default] hello world #2\n'] # no append by default!

    def test_log_file_append_mode(self, tmpdir):
        logfilefn = tmpdir.join('log_append.out')

        # The append mode is on by default, so we don't need to specify it for File
        py.log.setconsumer("default", py.log.Path(logfilefn, append=True,
                                                    buffering=0))
        assert logfilefn.check()
        py.log.Producer("default")("hello world #1")
        lines = logfilefn.readlines()
        assert lines == ['[default] hello world #1\n']
        py.log.setconsumer("default", py.log.Path(logfilefn, append=True,
                                                    buffering=0))
        py.log.Producer("default")("hello world #1")
        lines = logfilefn.readlines()
        assert lines == ['[default] hello world #1\n',
                         '[default] hello world #1\n']

    def test_log_file_delayed_create(self, tmpdir):
        logfilefn = tmpdir.join('log_create.out')

        py.log.setconsumer("default", py.log.Path(logfilefn,
                                        delayed_create=True, buffering=0))
        assert not logfilefn.check()
        py.log.Producer("default")("hello world #1")
        lines = logfilefn.readlines()
        assert lines == ['[default] hello world #1\n']

    def test_keyword_based_log_files(self, tmpdir):
        logfiles = []
        keywords = 'k1 k2 k3'.split()
        for key in keywords:
            path = tmpdir.join(key)
            py.log.setconsumer(key, py.log.Path(path, buffering=0))

        py.log.Producer('k1')('1')
        py.log.Producer('k2')('2')
        py.log.Producer('k3')('3')

        for key in keywords:
            path = tmpdir.join(key)
            assert path.read().strip() == '[%s] %s' % (key, key[-1])

    # disabled for now; the syslog log file can usually be read only by root
    # I manually inspected /var/log/messages and the entries were there
    def no_test_log_syslog(self):
        py.log.setconsumer("default", py.log.Syslog())
        py.log.default("hello world #1")

    # disabled for now until I figure out how to read entries in the
    # Event Logs on Windows
    # I manually inspected the Application Log and the entries were there
    def no_test_log_winevent(self):
        py.log.setconsumer("default", py.log.WinEvent())
        py.log.default("hello world #1")

    # disabled for now until I figure out how to properly pass the parameters
    def no_test_log_email(self):
        py.log.setconsumer("default", py.log.Email(mailhost="gheorghiu.net",
                                                   fromaddr="grig",
                                                   toaddrs="grig",
                                                   subject = "py.log email"))
        py.log.default("hello world #1")
