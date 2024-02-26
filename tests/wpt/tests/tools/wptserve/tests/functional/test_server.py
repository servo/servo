import os
import socket
import ssl
import unittest
from urllib.error import HTTPError

import pytest

from localpaths import repo_root

wptserve = pytest.importorskip("wptserve")
from .base import TestUsingH2Server, TestUsingServer, doc_root


class TestFileHandler(TestUsingServer):
    def test_not_handled(self):
        with self.assertRaises(HTTPError) as cm:
            self.request("/not_existing")

        self.assertEqual(cm.exception.code, 404)


class TestRewriter(TestUsingServer):
    def test_rewrite(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.request_path

        route = ("GET", "/test/rewritten", handler)
        self.server.rewriter.register("GET", "/test/original", route[1])
        self.server.router.register(*route)
        resp = self.request("/test/original")
        self.assertEqual(200, resp.getcode())
        self.assertEqual(b"/test/rewritten", resp.read())


class TestRequestHandler(TestUsingServer):
    def test_exception(self):
        @wptserve.handlers.handler
        def handler(request, response):
            raise Exception

        route = ("GET", "/test/raises", handler)
        self.server.router.register(*route)
        with self.assertRaises(HTTPError) as cm:
            self.request("/test/raises")

        self.assertEqual(cm.exception.code, 500)

    def test_many_headers(self):
        headers = {"X-Val%d" % i: str(i) for i in range(256)}

        @wptserve.handlers.handler
        def handler(request, response):
            # Additional headers are added by urllib.request.
            assert len(request.headers) > len(headers)
            for k, v in headers.items():
                assert request.headers.get(k) == \
                    wptserve.utils.isomorphic_encode(v)
            return "OK"

        route = ("GET", "/test/headers", handler)
        self.server.router.register(*route)
        resp = self.request("/test/headers", headers=headers)
        self.assertEqual(200, resp.getcode())


class TestH1TLSHandshake(TestUsingServer):
    def setUp(self):
        self.server = wptserve.server.WebTestHttpd(
            host="localhost",
            port=0,
            use_ssl=True,
            key_file=os.path.join(repo_root, "tools", "certs", "web-platform.test.key"),
            certificate=os.path.join(
                repo_root, "tools", "certs", "web-platform.test.pem"
            ),
            doc_root=doc_root,
        )
        self.server.start()

    def test_no_handshake(self):
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        context.load_verify_locations(
            os.path.join(repo_root, "tools", "certs", "cacert.pem")
        )

        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s_no_handshake:
            s_no_handshake.connect(("localhost", self.server.port))
            # Note: this socket is left open, notably not sending the TLS handshake.

            with socket.socket(socket.AF_INET, socket.SOCK_STREAM, 0) as sock:
                sock.settimeout(10)
                with context.wrap_socket(
                    sock,
                    do_handshake_on_connect=False,
                    server_hostname="web-platform.test",
                ) as ssock:
                    ssock.connect(("localhost", self.server.port))
                    ssock.do_handshake()
                    # The pass condition here is essentially "don't raise TimeoutError".


class TestH2TLSHandshake(TestUsingH2Server):
    def test_no_handshake(self):
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_CLIENT)
        context.load_verify_locations(
            os.path.join(repo_root, "tools", "certs", "cacert.pem")
        )

        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s_no_handshake:
            s_no_handshake.connect(("localhost", self.server.port))
            # Note: this socket is left open, notably not sending the TLS handshake.

            with socket.socket(socket.AF_INET, socket.SOCK_STREAM, 0) as sock:
                sock.settimeout(10)
                with context.wrap_socket(
                    sock,
                    do_handshake_on_connect=False,
                    server_hostname="web-platform.test",
                ) as ssock:
                    ssock.connect(("localhost", self.server.port))
                    ssock.do_handshake()
                    # The pass condition here is essentially "don't raise TimeoutError".


class TestH2Version(TestUsingH2Server):
    # The purpose of this test is to ensure that all TestUsingH2Server tests
    # actually end up using HTTP/2, in case there's any protocol negotiation.
    def test_http_version(self):
        resp = self.client.get('/')

        assert resp.http_version == 'HTTP/2'


class TestFileHandlerH2(TestUsingH2Server):
    def test_not_handled(self):
        resp = self.client.get("/not_existing")

        assert resp.status_code == 404


class TestRewriterH2(TestUsingH2Server):
    def test_rewrite(self):
        @wptserve.handlers.handler
        def handler(request, response):
            return request.request_path

        route = ("GET", "/test/rewritten", handler)
        self.server.rewriter.register("GET", "/test/original", route[1])
        self.server.router.register(*route)
        resp = self.client.get("/test/original")
        assert resp.status_code == 200
        assert resp.content == b"/test/rewritten"


class TestRequestHandlerH2(TestUsingH2Server):
    def test_exception(self):
        @wptserve.handlers.handler
        def handler(request, response):
            raise Exception

        route = ("GET", "/test/raises", handler)
        self.server.router.register(*route)
        resp = self.client.get("/test/raises")

        assert resp.status_code == 500

    def test_frame_handler_exception(self):
        class handler_cls:
            def frame_handler(self, request):
                raise Exception

        route = ("GET", "/test/raises", handler_cls())
        self.server.router.register(*route)
        resp = self.client.get("/test/raises")

        assert resp.status_code == 500


if __name__ == "__main__":
    unittest.main()
