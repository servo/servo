# mypy: allow-untyped-defs

import json
import sys
import traceback
import logging

from urllib.parse import parse_qsl

global logger
logger = logging.getLogger("wave-api-handler")


class ApiHandler:
    def __init__(self, web_root):
        self._web_root = web_root

    def set_headers(self, response, headers):
        if not isinstance(response.headers, list):
            response.headers = []
        for header in headers:
            response.headers.append(header)

    def send_json(self, data, response, status=None):
        if status is None:
            status = 200
        json_string = json.dumps(data, indent=4)
        response.content = json_string
        self.set_headers(response, [("Content-Type", "application/json")])
        response.status = status

    def send_file(self, blob, file_name, response):
        self.set_headers(response,
                         [("Content-Disposition",
                           "attachment;filename=" + file_name)])
        response.content = blob

    def send_zip(self, data, file_name, response):
        response.headers = [("Content-Type", "application/x-compressed")]
        self.send_file(data, file_name, response)

    def parse_uri(self, request):
        path = request.url_parts.path
        if self._web_root is not None:
            path = path[len(self._web_root):]

        uri_parts = list(filter(None, path.split("/")))
        return uri_parts

    def parse_query_parameters(self, request):
        return dict(parse_qsl(request.url_parts.query))

    def handle_exception(self, message):
        info = sys.exc_info()
        traceback.print_tb(info[2])
        logger.error(f"{message}: {info[0].__name__}: {info[1].args[0]}")

    def create_hal_list(self, items, uris, index, count, total):
        hal_list = {}
        links = {}
        if uris is not None:
            for relation in uris:
                if relation == "self":
                    continue
                uri = uris[relation]
                templated = "{" in uri
                links[relation] = {"href": uri, "templated": templated}

            if "self" in uris:
                self_uri = uris["self"]
                self_uri += f"?index={index}&count={count}"
                links["self"] = {"href": self_uri}

                first_uri = uris["self"]
                first_uri += f"?index={0}&count={count}"
                links["first"] = {"href": first_uri}

                last_uri = uris["self"]
                last_uri += f"?index={total - (total % count)}&count={count}"
                links["last"] = {"href": last_uri}

                if index + count <= total:
                    next_index = index + count
                    next_uri = uris["self"]
                    next_uri += f"?index={next_index}&count={count}"
                    links["next"] = {"href": next_uri}

                if index != 0:
                    previous_index = index - count
                    if previous_index < 0:
                        previous_index = 0
                    previous_uri = uris["self"]
                    previous_uri += f"?index={previous_index}&count={count}"
                    links["previous"] = {"href": previous_uri}

        hal_list["_links"] = links
        hal_list["items"] = items

        return hal_list
