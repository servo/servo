import os
import unittest
import time
import json

from six import assertRegex
from six.moves import urllib

import pytest

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingServer, doc_root


class TestStatus(TestUsingServer):
    def test_status(self):
        resp = self.request("/document.txt", query="pipe=status(202)")
        self.assertEqual(resp.getcode(), 202)

class TestHeader(TestUsingServer):
    def test_not_set(self):
        resp = self.request("/document.txt", query="pipe=header(X-TEST,PASS)")
        self.assertEqual(resp.info()["X-TEST"], "PASS")

    def test_set(self):
        resp = self.request("/document.txt", query="pipe=header(Content-Type,text/html)")
        self.assertEqual(resp.info()["Content-Type"], "text/html")

    def test_multiple(self):
        resp = self.request("/document.txt", query="pipe=header(X-Test,PASS)|header(Content-Type,text/html)")
        self.assertEqual(resp.info()["X-TEST"], "PASS")
        self.assertEqual(resp.info()["Content-Type"], "text/html")

    def test_multiple_same(self):
        resp = self.request("/document.txt", query="pipe=header(Content-Type,FAIL)|header(Content-Type,text/html)")
        self.assertEqual(resp.info()["Content-Type"], "text/html")

    def test_multiple_append(self):
        resp = self.request("/document.txt", query="pipe=header(X-Test,1)|header(X-Test,2,True)")
        self.assert_multiple_headers(resp, "X-Test", ["1", "2"])

    def test_semicolon(self):
        resp = self.request("/document.txt", query="pipe=header(Refresh,3;url=http://example.com)")
        self.assertEqual(resp.info()["Refresh"], "3;url=http://example.com")

    def test_escape_comma(self):
        resp = self.request("/document.txt", query=r"pipe=header(Expires,Thu\,%2014%20Aug%201986%2018:00:00%20GMT)")
        self.assertEqual(resp.info()["Expires"], "Thu, 14 Aug 1986 18:00:00 GMT")

    def test_escape_parenthesis(self):
        resp = self.request("/document.txt", query=r"pipe=header(User-Agent,Mozilla/5.0%20(X11;%20Linux%20x86_64;%20rv:12.0\)")
        self.assertEqual(resp.info()["User-Agent"], "Mozilla/5.0 (X11; Linux x86_64; rv:12.0)")

class TestSlice(TestUsingServer):
    def test_both_bounds(self):
        resp = self.request("/document.txt", query="pipe=slice(1,10)")
        expected = open(os.path.join(doc_root, "document.txt"), 'rb').read()
        self.assertEqual(resp.read(), expected[1:10])

    def test_no_upper(self):
        resp = self.request("/document.txt", query="pipe=slice(1)")
        expected = open(os.path.join(doc_root, "document.txt"), 'rb').read()
        self.assertEqual(resp.read(), expected[1:])

    def test_no_lower(self):
        resp = self.request("/document.txt", query="pipe=slice(null,10)")
        expected = open(os.path.join(doc_root, "document.txt"), 'rb').read()
        self.assertEqual(resp.read(), expected[:10])

class TestSub(TestUsingServer):
    def test_sub_config(self):
        resp = self.request("/sub.txt", query="pipe=sub")
        expected = b"localhost localhost %i" % self.server.port
        self.assertEqual(resp.read().rstrip(), expected)

    def test_sub_file_hash(self):
        resp = self.request("/sub_file_hash.sub.txt")
        expected = b"""
md5: JmI1W8fMHfSfCarYOSxJcw==
sha1: nqpWqEw4IW8NjD6R375gtrQvtTo=
sha224: RqQ6fMmta6n9TuA/vgTZK2EqmidqnrwBAmQLRQ==
sha256: G6Ljg1uPejQxqFmvFOcV/loqnjPTW5GSOePOfM/u0jw=
sha384: lkXHChh1BXHN5nT5BYhi1x67E1CyYbPKRKoF2LTm5GivuEFpVVYtvEBHtPr74N9E
sha512: r8eLGRTc7ZznZkFjeVLyo6/FyQdra9qmlYCwKKxm3kfQAswRS9+3HsYk3thLUhcFmmWhK4dXaICzJwGFonfXwg=="""
        self.assertEqual(resp.read().rstrip(), expected.strip())

    def test_sub_file_hash_unrecognized(self):
        with self.assertRaises(urllib.error.HTTPError):
            self.request("/sub_file_hash_unrecognized.sub.txt")

    def test_sub_headers(self):
        resp = self.request("/sub_headers.txt", query="pipe=sub", headers={"X-Test": "PASS"})
        expected = b"PASS"
        self.assertEqual(resp.read().rstrip(), expected)

    def test_sub_location(self):
        resp = self.request("/sub_location.sub.txt?query_string")
        expected = """
host: localhost:{0}
hostname: localhost
path: /sub_location.sub.txt
pathname: /sub_location.sub.txt
port: {0}
query: ?query_string
scheme: http
server: http://localhost:{0}""".format(self.server.port).encode("ascii")
        self.assertEqual(resp.read().rstrip(), expected.strip())

    def test_sub_params(self):
        resp = self.request("/sub_params.txt", query="plus+pct-20%20pct-3D%3D=PLUS+PCT-20%20PCT-3D%3D&pipe=sub")
        expected = b"PLUS PCT-20 PCT-3D="
        self.assertEqual(resp.read().rstrip(), expected)

    def test_sub_url_base(self):
        resp = self.request("/sub_url_base.sub.txt")
        self.assertEqual(resp.read().rstrip(), b"Before / After")

    def test_sub_url_base_via_filename_with_query(self):
        resp = self.request("/sub_url_base.sub.txt?pipe=slice(5,10)")
        self.assertEqual(resp.read().rstrip(), b"e / A")

    def test_sub_uuid(self):
        resp = self.request("/sub_uuid.sub.txt")
        assertRegex(self, resp.read().rstrip(), b"Before [a-f0-9-]+ After")

    def test_sub_var(self):
        resp = self.request("/sub_var.sub.txt")
        port = self.server.port
        expected = b"localhost %d A %d B localhost C" % (port, port)
        self.assertEqual(resp.read().rstrip(), expected)

    def test_sub_fs_path(self):
        resp = self.request("/subdir/sub_path.sub.txt")
        root = os.path.abspath(doc_root)
        expected = """%(root)s%(sep)ssubdir%(sep)ssub_path.sub.txt
%(root)s%(sep)ssub_path.sub.txt
%(root)s%(sep)ssub_path.sub.txt
""" % {"root": root, "sep": os.path.sep}
        self.assertEqual(resp.read(), expected.encode("utf8"))

    def test_sub_header_or_default(self):
        resp = self.request("/sub_header_or_default.sub.txt", headers={"X-Present": "OK"})
        expected = b"OK\nabsent-default"
        self.assertEqual(resp.read().rstrip(), expected)

class TestTrickle(TestUsingServer):
    def test_trickle(self):
        #Actually testing that the response trickles in is not that easy
        t0 = time.time()
        resp = self.request("/document.txt", query="pipe=trickle(1:d2:5:d1:r2)")
        t1 = time.time()
        expected = open(os.path.join(doc_root, "document.txt"), 'rb').read()
        self.assertEqual(resp.read(), expected)
        self.assertGreater(6, t1-t0)

    def test_headers(self):
        resp = self.request("/document.txt", query="pipe=trickle(d0.01)")
        self.assertEqual(resp.info()["Cache-Control"], "no-cache, no-store, must-revalidate")
        self.assertEqual(resp.info()["Pragma"], "no-cache")
        self.assertEqual(resp.info()["Expires"], "0")

class TestPipesWithVariousHandlers(TestUsingServer):
    def test_with_python_file_handler(self):
        resp = self.request("/test_string.py", query="pipe=slice(null,2)")
        self.assertEqual(resp.read(), b"PA")

    def test_with_python_func_handler(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return "PASS"
        route = ("GET", "/test/test_pipes_1/", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], query="pipe=slice(null,2)")
        self.assertEqual(resp.read(), b"PA")

    def test_with_python_func_handler_using_response_writer(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_content("PASS")
        route = ("GET", "/test/test_pipes_1/", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], query="pipe=slice(null,2)")
        # slice has not been applied to the response, because response.writer was used.
        self.assertEqual(resp.read(), b"PASS")

    def test_header_pipe_with_python_func_using_response_writer(self):
        @wptserve.handlers.handler
        def handler(request, response):
            response.writer.write_content("CONTENT")
        route = ("GET", "/test/test_pipes_1/", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], query="pipe=header(X-TEST,FAIL)")
        # header pipe was ignored, because response.writer was used.
        self.assertFalse(resp.info().get("X-TEST"))
        self.assertEqual(resp.read(), b"CONTENT")

    def test_with_json_handler(self):
        @wptserve.handlers.json_handler
        def handler(request, response):
            return json.dumps({'data': 'PASS'})
        route = ("GET", "/test/test_pipes_2/", handler)
        self.server.router.register(*route)
        resp = self.request(route[1], query="pipe=slice(null,2)")
        self.assertEqual(resp.read(), b'"{')

    def test_slice_with_as_is_handler(self):
        resp = self.request("/test.asis", query="pipe=slice(null,2)")
        self.assertEqual(202, resp.getcode())
        self.assertEqual("Giraffe", resp.msg)
        self.assertEqual("PASS", resp.info()["X-Test"])
        # slice has not been applied to the response, because response.writer was used.
        self.assertEqual(b"Content", resp.read())

    def test_headers_with_as_is_handler(self):
        resp = self.request("/test.asis", query="pipe=header(X-TEST,FAIL)")
        self.assertEqual(202, resp.getcode())
        self.assertEqual("Giraffe", resp.msg)
        # header pipe was ignored.
        self.assertEqual("PASS", resp.info()["X-TEST"])
        self.assertEqual(b"Content", resp.read())

    def test_trickle_with_as_is_handler(self):
        t0 = time.time()
        resp = self.request("/test.asis", query="pipe=trickle(1:d2:5:d1:r2)")
        t1 = time.time()
        self.assertTrue(b'Content' in resp.read())
        self.assertGreater(6, t1-t0)

if __name__ == '__main__':
    unittest.main()
