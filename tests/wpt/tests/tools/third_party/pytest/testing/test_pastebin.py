import io
from typing import List
from typing import Union

import pytest
from _pytest.monkeypatch import MonkeyPatch
from _pytest.pytester import Pytester


class TestPasteCapture:
    @pytest.fixture
    def pastebinlist(self, monkeypatch, request) -> List[Union[str, bytes]]:
        pastebinlist: List[Union[str, bytes]] = []
        plugin = request.config.pluginmanager.getplugin("pastebin")
        monkeypatch.setattr(plugin, "create_new_paste", pastebinlist.append)
        return pastebinlist

    def test_failed(self, pytester: Pytester, pastebinlist) -> None:
        testpath = pytester.makepyfile(
            """
            import pytest
            def test_pass() -> None:
                pass
            def test_fail():
                assert 0
            def test_skip():
                pytest.skip("")
        """
        )
        reprec = pytester.inline_run(testpath, "--pastebin=failed")
        assert len(pastebinlist) == 1
        s = pastebinlist[0]
        assert s.find("def test_fail") != -1
        assert reprec.countoutcomes() == [1, 1, 1]

    def test_all(self, pytester: Pytester, pastebinlist) -> None:
        from _pytest.pytester import LineMatcher

        testpath = pytester.makepyfile(
            """
            import pytest
            def test_pass():
                pass
            def test_fail():
                assert 0
            def test_skip():
                pytest.skip("")
        """
        )
        reprec = pytester.inline_run(testpath, "--pastebin=all", "-v")
        assert reprec.countoutcomes() == [1, 1, 1]
        assert len(pastebinlist) == 1
        contents = pastebinlist[0].decode("utf-8")
        matcher = LineMatcher(contents.splitlines())
        matcher.fnmatch_lines(
            [
                "*test_pass PASSED*",
                "*test_fail FAILED*",
                "*test_skip SKIPPED*",
                "*== 1 failed, 1 passed, 1 skipped in *",
            ]
        )

    def test_non_ascii_paste_text(self, pytester: Pytester, pastebinlist) -> None:
        """Make sure that text which contains non-ascii characters is pasted
        correctly. See #1219.
        """
        pytester.makepyfile(
            test_unicode="""\
            def test():
                assert '☺' == 1
            """
        )
        result = pytester.runpytest("--pastebin=all")
        expected_msg = "*assert '☺' == 1*"
        result.stdout.fnmatch_lines(
            [
                expected_msg,
                "*== 1 failed in *",
                "*Sending information to Paste Service*",
            ]
        )
        assert len(pastebinlist) == 1


class TestPaste:
    @pytest.fixture
    def pastebin(self, request):
        return request.config.pluginmanager.getplugin("pastebin")

    @pytest.fixture
    def mocked_urlopen_fail(self, monkeypatch: MonkeyPatch):
        """Monkeypatch the actual urlopen call to emulate a HTTP Error 400."""
        calls = []

        import urllib.error
        import urllib.request

        def mocked(url, data):
            calls.append((url, data))
            raise urllib.error.HTTPError(url, 400, "Bad request", {}, io.BytesIO())

        monkeypatch.setattr(urllib.request, "urlopen", mocked)
        return calls

    @pytest.fixture
    def mocked_urlopen_invalid(self, monkeypatch: MonkeyPatch):
        """Monkeypatch the actual urlopen calls done by the internal plugin
        function that connects to bpaste service, but return a url in an
        unexpected format."""
        calls = []

        def mocked(url, data):
            calls.append((url, data))

            class DummyFile:
                def read(self):
                    # part of html of a normal response
                    return b'View <a href="/invalid/3c0c6750bd">raw</a>.'

            return DummyFile()

        import urllib.request

        monkeypatch.setattr(urllib.request, "urlopen", mocked)
        return calls

    @pytest.fixture
    def mocked_urlopen(self, monkeypatch: MonkeyPatch):
        """Monkeypatch the actual urlopen calls done by the internal plugin
        function that connects to bpaste service."""
        calls = []

        def mocked(url, data):
            calls.append((url, data))

            class DummyFile:
                def read(self):
                    # part of html of a normal response
                    return b'View <a href="/raw/3c0c6750bd">raw</a>.'

            return DummyFile()

        import urllib.request

        monkeypatch.setattr(urllib.request, "urlopen", mocked)
        return calls

    def test_pastebin_invalid_url(self, pastebin, mocked_urlopen_invalid) -> None:
        result = pastebin.create_new_paste(b"full-paste-contents")
        assert (
            result
            == "bad response: invalid format ('View <a href=\"/invalid/3c0c6750bd\">raw</a>.')"
        )
        assert len(mocked_urlopen_invalid) == 1

    def test_pastebin_http_error(self, pastebin, mocked_urlopen_fail) -> None:
        result = pastebin.create_new_paste(b"full-paste-contents")
        assert result == "bad response: HTTP Error 400: Bad request"
        assert len(mocked_urlopen_fail) == 1

    def test_create_new_paste(self, pastebin, mocked_urlopen) -> None:
        result = pastebin.create_new_paste(b"full-paste-contents")
        assert result == "https://bpa.st/show/3c0c6750bd"
        assert len(mocked_urlopen) == 1
        url, data = mocked_urlopen[0]
        assert type(data) is bytes
        lexer = "text"
        assert url == "https://bpa.st"
        assert "lexer=%s" % lexer in data.decode()
        assert "code=full-paste-contents" in data.decode()
        assert "expiry=1week" in data.decode()

    def test_create_new_paste_failure(self, pastebin, monkeypatch: MonkeyPatch) -> None:
        import io
        import urllib.request

        def response(url, data):
            stream = io.BytesIO(b"something bad occurred")
            return stream

        monkeypatch.setattr(urllib.request, "urlopen", response)
        result = pastebin.create_new_paste(b"full-paste-contents")
        assert result == "bad response: invalid format ('something bad occurred')"
