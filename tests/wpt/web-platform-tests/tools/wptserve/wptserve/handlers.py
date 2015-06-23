import cgi
import json
import os
import traceback
import urllib
import urlparse

from constants import content_types
from pipes import Pipeline, template
from ranges import RangeParser
from request import Authentication
from response import MultipartContent
from utils import HTTPException

__all__ = ["file_handler", "python_script_handler",
           "FunctionHandler", "handler", "json_handler",
           "as_is_handler", "ErrorHandler", "BasicAuthHandler"]


def guess_content_type(path):
    ext = os.path.splitext(path)[1].lstrip(".")
    if ext in content_types:
        return content_types[ext]

    return "application/octet-stream"



def filesystem_path(base_path, request, url_base="/"):
    if base_path is None:
        base_path = request.doc_root

    path = request.url_parts.path

    if path.startswith(url_base):
        path = path[len(url_base):]

    if ".." in path:
        raise HTTPException(404)

    new_path = os.path.join(base_path, path)

    # Otherwise setting path to / allows access outside the root directory
    if not new_path.startswith(base_path):
        raise HTTPException(404)

    return new_path

class DirectoryHandler(object):
    def __init__(self, base_path=None, url_base="/"):
        self.base_path = base_path
        self.url_base = url_base

    def __call__(self, request, response):
        if not request.url_parts.path.endswith("/"):
            raise HTTPException(404)

        path = filesystem_path(self.base_path, request, self.url_base)

        assert os.path.isdir(path)

        response.headers = [("Content-Type", "text/html")]
        response.content = """<!doctype html>
<meta name="viewport" content="width=device-width">
<title>Directory listing for %(path)s</title>
<h1>Directory listing for %(path)s</h1>
<ul>
%(items)s
</li>
""" % {"path": cgi.escape(request.url_parts.path),
       "items": "\n".join(self.list_items(request, path))}

    def list_items(self, request, path):
        # TODO: this won't actually list all routes, only the
        # ones that correspond to a real filesystem path. It's
        # not possible to list every route that will match
        # something, but it should be possible to at least list the
        # statically defined ones
        base_path = request.url_parts.path

        if not base_path.endswith("/"):
            base_path += "/"
        if base_path != "/":
            link = urlparse.urljoin(base_path, "..")
            yield ("""<li class="dir"><a href="%(link)s">%(name)s</a>""" %
                   {"link": link, "name": ".."})
        for item in sorted(os.listdir(path)):
            link = cgi.escape(urllib.quote(item))
            if os.path.isdir(os.path.join(path, item)):
                link += "/"
                class_ = "dir"
            else:
                class_ = "file"
            yield ("""<li class="%(class)s"><a href="%(link)s">%(name)s</a>""" %
                   {"link": link, "name": cgi.escape(item), "class": class_})


directory_handler = DirectoryHandler()


class FileHandler(object):
    def __init__(self, base_path=None, url_base="/"):
        self.base_path = base_path
        self.url_base = url_base
        self.directory_handler = DirectoryHandler(self.base_path)

    def __call__(self, request, response):
        path = filesystem_path(self.base_path, request, self.url_base)

        if os.path.isdir(path):
            return self.directory_handler(request, response)
        try:
            #This is probably racy with some other process trying to change the file
            file_size = os.stat(path).st_size
            response.headers.update(self.get_headers(request, path))
            if "Range" in request.headers:
                try:
                    byte_ranges = RangeParser()(request.headers['Range'], file_size)
                except HTTPException as e:
                    if e.code == 416:
                        response.headers.set("Content-Range", "bytes */%i" % file_size)
                        raise
            else:
                byte_ranges = None
            data = self.get_data(response, path, byte_ranges)
            response.content = data
            query = urlparse.parse_qs(request.url_parts.query)

            pipeline = None
            if "pipe" in query:
                pipeline = Pipeline(query["pipe"][-1])
            elif os.path.splitext(path)[0].endswith(".sub"):
                pipeline = Pipeline("sub")
            if pipeline is not None:
                response = pipeline(request, response)

            return response

        except (OSError, IOError):
            raise HTTPException(404)

    def get_headers(self, request, path):
        rv = self.default_headers(path)
        rv.extend(self.load_headers(request, os.path.join(os.path.split(path)[0], "__dir__")))
        rv.extend(self.load_headers(request, path))
        return rv

    def load_headers(self, request, path):
        headers_path = path + ".sub.headers"
        if os.path.exists(headers_path):
            use_sub = True
        else:
            headers_path = path + ".headers"
            use_sub = False

        try:
            with open(headers_path) as headers_file:
                data = headers_file.read()
        except IOError:
            return []
        else:
            if use_sub:
                data = template(request, data)
            return [tuple(item.strip() for item in line.split(":", 1))
                    for line in data.splitlines() if line]

    def get_data(self, response, path, byte_ranges):
        with open(path, 'rb') as f:
            if byte_ranges is None:
                return f.read()
            else:
                response.status = 206
                if len(byte_ranges) > 1:
                    parts_content_type, content = self.set_response_multipart(response,
                                                                              byte_ranges,
                                                                              f)
                    for byte_range in byte_ranges:
                        content.append_part(self.get_range_data(f, byte_range),
                                            parts_content_type,
                                            [("Content-Range", byte_range.header_value())])
                    return content
                else:
                    response.headers.set("Content-Range", byte_ranges[0].header_value())
                    return self.get_range_data(f, byte_ranges[0])

    def set_response_multipart(self, response, ranges, f):
        parts_content_type = response.headers.get("Content-Type")
        if parts_content_type:
            parts_content_type = parts_content_type[-1]
        else:
            parts_content_type = None
        content = MultipartContent()
        response.headers.set("Content-Type", "multipart/byteranges; boundary=%s" % content.boundary)
        return parts_content_type, content

    def get_range_data(self, f, byte_range):
        f.seek(byte_range.lower)
        return f.read(byte_range.upper - byte_range.lower)

    def default_headers(self, path):
        return [("Content-Type", guess_content_type(path))]


file_handler = FileHandler()


class PythonScriptHandler(object):
    def __init__(self, base_path=None, url_base="/"):
        self.base_path = base_path
        self.url_base = url_base

    def __call__(self, request, response):
        path = filesystem_path(self.base_path, request, self.url_base)

        try:
            environ = {"__file__": path}
            execfile(path, environ, environ)
            if "main" in environ:
                handler = FunctionHandler(environ["main"])
                handler(request, response)
            else:
                raise HTTPException(500, "No main function in script %s" % path)
        except IOError:
            raise HTTPException(404)

python_script_handler = PythonScriptHandler()

class FunctionHandler(object):
    def __init__(self, func):
        self.func = func

    def __call__(self, request, response):
        try:
            rv = self.func(request, response)
        except Exception:
            msg = traceback.format_exc()
            raise HTTPException(500, message=msg)
        if rv is not None:
            if isinstance(rv, tuple):
                if len(rv) == 3:
                    status, headers, content = rv
                    response.status = status
                elif len(rv) == 2:
                    headers, content = rv
                else:
                    raise HTTPException(500)
                response.headers.update(headers)
            else:
                content = rv
            response.content = content


#The generic name here is so that this can be used as a decorator
def handler(func):
    return FunctionHandler(func)


class JsonHandler(object):
    def __init__(self, func):
        self.func = func

    def __call__(self, request, response):
        return FunctionHandler(self.handle_request)(request, response)

    def handle_request(self, request, response):
        rv = self.func(request, response)
        response.headers.set("Content-Type", "application/json")
        enc = json.dumps
        if isinstance(rv, tuple):
            rv = list(rv)
            value = tuple(rv[:-1] + [enc(rv[-1])])
            length = len(value[-1])
        else:
            value = enc(rv)
            length = len(value)
        response.headers.set("Content-Length", length)
        return value

def json_handler(func):
    return JsonHandler(func)

class AsIsHandler(object):
    def __init__(self, base_path=None, url_base="/"):
        self.base_path = base_path
        self.url_base = url_base

    def __call__(self, request, response):
        path = filesystem_path(self.base_path, request, self.url_base)

        try:
            with open(path) as f:
                response.writer.write_content(f.read())
            response.close_connection = True
        except IOError:
            raise HTTPException(404)

as_is_handler = AsIsHandler()

class BasicAuthHandler(object):
    def __init__(self, handler, user, password):
        """
         A Basic Auth handler

         :Args:
         - handler: a secondary handler for the request after authentication is successful (example file_handler)
         - user: string of the valid user name or None if any / all credentials are allowed
         - password: string of the password required
        """
        self.user = user
        self.password = password
        self.handler = handler

    def __call__(self, request, response):
        if "authorization" not in request.headers:
            response.status = 401
            response.headers.set("WWW-Authenticate", "Basic")
            return response
        else:
            auth = Authentication(request.headers)
            if self.user is not None and (self.user != auth.username or self.password != auth.password):
                response.set_error(403, "Invalid username or password")
                return response
            return self.handler(request, response)

basic_auth_handler = BasicAuthHandler(file_handler, None, None)

class ErrorHandler(object):
    def __init__(self, status):
        self.status = status

    def __call__(self, request, response):
        response.set_error(self.status)
