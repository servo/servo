import json
import os
import unittest
import urllib2
import uuid

import wptserve
from base import TestUsingServer, doc_root

class TestFileHandler(TestUsingServer):
    def test_GET(self):
        resp = self.request("/document.txt")
        self.assertEquals(200, resp.getcode())
        self.assertEquals("text/plain", resp.info()["Content-Type"])
        self.assertEquals(open(os.path.join(doc_root, "document.txt")).read(), resp.read())

    def test_headers(self):
        resp = self.request("/with_headers.txt")
        self.assertEquals(200, resp.getcode())
        self.assertEquals("PASS", resp.info()["Custom-Header"])
        # This will fail if it isn't a valid uuid
        uuid.UUID(resp.info()["Another-Header"])
        self.assertEquals(resp.info()["Same-Value-Header"], resp.info()["Another-Header"])


    def test_range(self):
        resp = self.request("/document.txt", headers={"Range":"bytes=10-19"})
        self.assertEquals(206, resp.getcode())
        data = resp.read()
        expected = open(os.path.join(doc_root, "document.txt")).read()
        self.assertEquals(10, len(data))
        self.assertEquals("bytes 10-19/%i" % len(expected), resp.info()['Content-Range'])
        self.assertEquals("10", resp.info()['Content-Length'])
        self.assertEquals(expected[10:20], data)

    def test_range_no_end(self):
        resp = self.request("/document.txt", headers={"Range":"bytes=10-"})
        self.assertEquals(206, resp.getcode())
        data = resp.read()
        expected = open(os.path.join(doc_root, "document.txt")).read()
        self.assertEquals(len(expected) - 10, len(data))
        self.assertEquals("bytes 10-%i/%i" % (len(expected) - 1, len(expected)), resp.info()['Content-Range'])
        self.assertEquals(expected[10:], data)

    def test_range_no_start(self):
        resp = self.request("/document.txt", headers={"Range":"bytes=-10"})
        self.assertEquals(206, resp.getcode())
        data = resp.read()
        expected = open(os.path.join(doc_root, "document.txt")).read()
        self.assertEquals(10, len(data))
        self.assertEquals("bytes %i-%i/%i" % (len(expected) - 10,
                                              len(expected) - 1,
                                              len(expected)), resp.info()['Content-Range'])
        self.assertEquals(expected[-10:], data)

    def test_multiple_ranges(self):
        resp = self.request("/document.txt", headers={"Range":"bytes=1-2,5-7,6-10"})
        self.assertEquals(206, resp.getcode())
        data = resp.read()
        expected = open(os.path.join(doc_root, "document.txt")).read()
        self.assertTrue(resp.info()["Content-Type"].startswith("multipart/byteranges; boundary="))
        boundary = resp.info()["Content-Type"].split("boundary=")[1]
        parts = data.split("--" + boundary)
        self.assertEquals("\r\n", parts[0])
        self.assertEquals("--", parts[-1])
        expected_parts = [("1-2", expected[1:3]), ("5-10", expected[5:11])]
        for expected_part, part in zip(expected_parts, parts[1:-1]):
            header_string, body = part.split("\r\n\r\n")
            headers = dict(item.split(": ", 1) for item in header_string.split("\r\n") if item.strip())
            self.assertEquals(headers["Content-Type"], "text/plain")
            self.assertEquals(headers["Content-Range"], "bytes %s/%i" % (expected_part[0], len(expected)))
            self.assertEquals(expected_part[1] + "\r\n", body)

    def test_range_invalid(self):
        with self.assertRaises(urllib2.HTTPError) as cm:
            self.request("/document.txt", headers={"Range":"bytes=11-10"})
        self.assertEquals(cm.exception.code, 416)

        expected = open(os.path.join(doc_root, "document.txt")).read()
        with self.assertRaises(urllib2.HTTPError) as cm:
            self.request("/document.txt", headers={"Range":"bytes=%i-%i" % (len(expected), len(expected) + 10)})
        self.assertEquals(cm.exception.code, 416)


class TestFunctionHandler(TestUsingServer):
    def test_string_rv(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return "test data"

        route = ("GET", "/test/test_string_rv", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        self.assertEquals(200, resp.getcode())
        self.assertEquals("9", resp.info()["Content-Length"])
        self.assertEquals("test data", resp.read())

    def test_tuple_2_rv(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return [("Content-Length", 4), ("test-header", "test-value")], "test data"

        route = ("GET", "/test/test_tuple_2_rv", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        self.assertEquals(200, resp.getcode())
        self.assertEquals("4", resp.info()["Content-Length"])
        self.assertEquals("test-value", resp.info()["test-header"])
        self.assertEquals("test", resp.read())

    def test_tuple_3_rv(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return 202, [("test-header", "test-value")], "test data"

        route = ("GET", "/test/test_tuple_3_rv", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        self.assertEquals(202, resp.getcode())
        self.assertEquals("test-value", resp.info()["test-header"])
        self.assertEquals("test data", resp.read())

    def test_tuple_3_rv_1(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return (202, "Some Status"), [("test-header", "test-value")], "test data"

        route = ("GET", "/test/test_tuple_3_rv_1", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        self.assertEquals(202, resp.getcode())
        self.assertEquals("Some Status", resp.msg)
        self.assertEquals("test-value", resp.info()["test-header"])
        self.assertEquals("test data", resp.read())

class TestJSONHandler(TestUsingServer):
    def test_json_0(self):
        @wptserve.handlers.json_handler
        def handler(request, response):
            return {"data": "test data"}

        route = ("GET", "/test/test_json_0", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        self.assertEquals(200, resp.getcode())
        self.assertEquals({"data": "test data"}, json.load(resp))

    def test_json_tuple_2(self):
        @wptserve.handlers.json_handler
        def handler(request, response):
            return [("Test-Header", "test-value")], {"data": "test data"}

        route = ("GET", "/test/test_json_tuple_2", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        self.assertEquals(200, resp.getcode())
        self.assertEquals("test-value", resp.info()["test-header"])
        self.assertEquals({"data": "test data"}, json.load(resp))

    def test_json_tuple_3(self):
        @wptserve.handlers.json_handler
        def handler(request, response):
            return (202, "Giraffe"), [("Test-Header", "test-value")], {"data": "test data"}

        route = ("GET", "/test/test_json_tuple_2", handler)
        self.server.router.register(*route)
        resp = self.request(route[1])
        self.assertEquals(202, resp.getcode())
        self.assertEquals("Giraffe", resp.msg)
        self.assertEquals("test-value", resp.info()["test-header"])
        self.assertEquals({"data": "test data"}, json.load(resp))

class TestPythonHandler(TestUsingServer):
    def test_string(self):
        resp = self.request("/test_string.py")
        self.assertEquals(200, resp.getcode())
        self.assertEquals("text/plain", resp.info()["Content-Type"])
        self.assertEquals("PASS", resp.read())

    def test_tuple_2(self):
        resp = self.request("/test_tuple_2.py")
        self.assertEquals(200, resp.getcode())
        self.assertEquals("text/html", resp.info()["Content-Type"])
        self.assertEquals("PASS", resp.info()["X-Test"])
        self.assertEquals("PASS", resp.read())

    def test_tuple_3(self):
        resp = self.request("/test_tuple_3.py")
        self.assertEquals(202, resp.getcode())
        self.assertEquals("Giraffe", resp.msg)
        self.assertEquals("text/html", resp.info()["Content-Type"])
        self.assertEquals("PASS", resp.info()["X-Test"])
        self.assertEquals("PASS", resp.read())

class TestDirectoryHandler(TestUsingServer):
    def test_directory(self):
        resp = self.request("/")
        self.assertEquals(200, resp.getcode())
        self.assertEquals("text/html", resp.info()["Content-Type"])
        #Add a check that the response is actually sane

class TestAsIsHandler(TestUsingServer):
    def test_as_is(self):
        resp = self.request("/test.asis")
        self.assertEquals(202, resp.getcode())
        self.assertEquals("Giraffe", resp.msg)
        self.assertEquals("PASS", resp.info()["X-Test"])
        self.assertEquals("Content", resp.read())
        #Add a check that the response is actually sane

if __name__ == '__main__':
    unittest.main()
