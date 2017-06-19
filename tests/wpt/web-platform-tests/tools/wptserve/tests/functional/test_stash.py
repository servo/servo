import unittest
import uuid

import pytest

wptserve = pytest.importorskip("wptserve")
from wptserve.router import any_method
from wptserve.stash import StashServer
from .base import TestUsingServer


class TestResponseSetCookie(TestUsingServer):
    def run(self, result=None):
        with StashServer(None, authkey=str(uuid.uuid4())):
            super(TestResponseSetCookie, self).run(result)

    def test_put_take(self):
        @wptserve.handlers.handler
        def handler(request, response):
            if request.method == "POST":
                request.server.stash.put(request.POST.first("id"), request.POST.first("data"))
                data = "OK"
            elif request.method == "GET":
                data = request.server.stash.take(request.GET.first("id"))
                if data is None:
                    return "NOT FOUND"
            return data

        id = str(uuid.uuid4())
        route = (any_method, "/test/put_take", handler)
        self.server.router.register(*route)

        resp = self.request(route[1], method="POST", body={"id": id, "data": "Sample data"})
        self.assertEqual(resp.read(), "OK")

        resp = self.request(route[1], query="id=" + id)
        self.assertEqual(resp.read(), "Sample data")

        resp = self.request(route[1], query="id=" + id)
        self.assertEqual(resp.read(), "NOT FOUND")


if __name__ == '__main__':
    unittest.main()
