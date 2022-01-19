# protocol-server
#
# a reference implementation of the Web Annotation Protocol
#
# Developed by Benjamin Young (@bigbulehat) and Shane McCarron (@halindrome).
# Sponsored by Spec-Ops (https://spec-ops.io)

from __future__ import print_function

import os
import sys

here = os.path.abspath(os.path.dirname(__file__))
repo_root = os.path.abspath(os.path.join(here, os.pardir, os.pardir))

sys.path.insert(0, os.path.join(repo_root, "tools"))
sys.path.insert(0, os.path.join(repo_root, "tools", "six"))
sys.path.insert(0, os.path.join(repo_root, "tools", "html5lib"))
sys.path.insert(0, os.path.join(repo_root, "tools", "wptserve"))
sys.path.insert(0, os.path.join(repo_root, "tools", "pywebsocket", "src"))
sys.path.insert(0, os.path.join(repo_root, "tools", "py"))
sys.path.insert(0, os.path.join(repo_root, "tools", "pytest"))
sys.path.insert(0, os.path.join(repo_root, "tools", "webdriver"))

import hashlib
import json
import urlparse
import uuid

import wptserve

myprotocol = 'http'
myhost = 'localhost'
port = 8080
doc_root = os.path.join(repo_root, "annotation-protocol", "files", "")
container_path = doc_root + 'annotations/'

URIroot = myprotocol + '://' + myhost + ':{0}'.format(port)

per_page = 10

MEDIA_TYPE = 'application/ld+json; profile="http://www.w3.org/ns/anno.jsonld"'
# Prefer header variants
PREFER_MINIMAL_CONTAINER = "http://www.w3.org/ns/ldp#PreferMinimalContainer"
PREFER_CONTAINED_IRIS = "http://www.w3.org/ns/oa#PreferContainedIRIs"
PREFER_CONTAINED_DESCRIPTIONS = \
        "http://www.w3.org/ns/oa#PreferContainedDescriptions"


# dictionary for annotations that we create on the fly
tempAnnotations = {}

def extract_preference(prefer):
    """Extracts the parameters from a Prefer header's value
    >>> extract_preferences('return=representation;include="http://www.w3.org/ns/ldp#PreferMinimalContainer http://www.w3.org/ns/oa#PreferContainedIRIs"')
    {"return": "representation", "include": ["http://www.w3.org/ns/ldp#PreferMinimalContainer", "http://www.w3.org/ns/oa#PreferContainedIRIs"]}
    """
    obj = {}
    if prefer:
        params = prefer.split(';')
        for p in params:
            key, value = p.split('=')
            obj[key] = value.strip('"').split(' ')
    return obj


def dump_json(obj):
    return json.dumps(obj, indent=4, sort_keys=True)

def add_cors_headers(resp):
    headers_file = doc_root + 'annotations/cors.headers'
    resp.headers.update(load_headers_from_file(headers_file))

def load_headers_from_file(path):
    headers = []
    with open(path, 'r') as header_file:
        data = header_file.read()
        headers = [tuple(item.strip() for item in line.split(":", 1))
                   for line in data.splitlines() if line]
    return headers

def annotation_files():
    files = []
    for file in os.listdir(container_path):
        if file.endswith('.jsonld') or file.endswith('.json'):
            files.append(file)
    for item in list(tempAnnotations.keys()):
        files.append(item)
    return files


def annotation_iris(skip=0):
    iris = []
    for filename in annotation_files():
        iris.append(URIroot + '/annotations/' + filename)
    return iris[skip:][:per_page]


def annotations(skip=0):
    annotations = []
    files = annotation_files()
    for file in files:
        if file.startswith("temp-"):
            annotations.append(json.loads(tempAnnotations[file]))
        else:
            with open(container_path + file, 'r') as annotation:
                annotations.append(json.load(annotation))
    return annotations


def total_annotations():
    return len(annotation_files())


@wptserve.handlers.handler
def collection_get(request, response):
    """Annotation Collection handler. NOTE: This also routes paging requests"""

    # Paginate if requested
    qs = urlparse.parse_qs(request.url_parts.query)
    if 'page' in qs:
        return page(request, response)

    # stub collection
    collection_json = {
      "@context": [
        "http://www.w3.org/ns/anno.jsonld",
        "http://www.w3.org/ns/ldp.jsonld"
      ],
      "id": URIroot + "/annotations/",
      "type": ["BasicContainer", "AnnotationCollection"],
      "total": 0,
      "label": "A Container for Web Annotations",
      "first": URIroot + "/annotations/?page=0"
    }

    last_page = (total_annotations() / per_page) - 1
    collection_json['last'] = URIroot + "/annotations/?page={0}".format(last_page)

    # Default Container format SHOULD be PreferContainedDescriptions
    preference = extract_preference(request.headers.get('Prefer'))
    if 'include' in preference:
        preference = preference['include']
    else:
        preference = None

    collection_json['total'] = total_annotations()
    # TODO: calculate last page and add it's page number

    if (qs.get('iris') and qs.get('iris')[0] is '1') \
            or (preference and PREFER_CONTAINED_IRIS in preference):
        return_iris = True
    else:
        return_iris = False

    # only PreferContainedIRIs has unqiue content
    if return_iris:
        collection_json['id'] += '?iris=1'
        collection_json['first'] += '&iris=1'
        collection_json['last'] += '&iris=1'

    if preference and PREFER_MINIMAL_CONTAINER not in preference:
        if return_iris:
            collection_json['first'] = annotation_iris()
        else:
            collection_json['first'] = annotations()

    collection_headers_file = doc_root + 'annotations/collection.headers'
    add_cors_headers(response)
    response.headers.update(load_headers_from_file(collection_headers_file))
    # this one's unique per request
    response.headers.set('Content-Location', collection_json['id'])
    return dump_json(collection_json)


@wptserve.handlers.handler
def collection_head(request, response):
    container_path = doc_root + request.request_path
    if os.path.isdir(container_path):
        response.status = 200
    else:
        response.status = 404

    add_cors_headers(response)
    headers_file = doc_root + 'annotations/collection.headers'
    for header, value in load_headers_from_file(headers_file):
        response.headers.append(header, value)

    response.content = None


@wptserve.handlers.handler
def collection_options(request, response):
    container_path = doc_root + request.request_path
    if os.path.isdir(container_path):
        response.status = 200
    else:
        response.status = 404

    add_cors_headers(response)
    headers_file = doc_root + 'annotations/collection.options.headers'
    for header, value in load_headers_from_file(headers_file):
        response.headers.append(header, value)

def page(request, response):
    page_json = {
      "@context": "http://www.w3.org/ns/anno.jsonld",
      "id": URIroot + "/annotations/",
      "type": "AnnotationPage",
      "partOf": {
        "id": URIroot + "/annotations/",
        "total": 42023
      },
      "next": URIroot + "/annotations/",
      "items": [
      ]
    }

    add_cors_headers(response)
    headers_file = doc_root + 'annotations/collection.headers'
    response.headers.update(load_headers_from_file(headers_file))

    qs = urlparse.parse_qs(request.url_parts.query)
    page_num = int(qs.get('page')[0])
    page_json['id'] += '?page={0}'.format(page_num)

    total = total_annotations()
    so_far = (per_page * (page_num+1))
    remaining = total - so_far

    if page_num != 0:
        page_json['prev'] = URIroot + '/annotations/?page={0}'.format(page_num-1)

    page_json['partOf']['total'] = total

    if remaining > per_page:
        page_json['next'] += '?page={0}'.format(page_num+1)
    else:
        del page_json['next']

    if qs.get('iris') and qs.get('iris')[0] is '1':
        page_json['items'] = annotation_iris(so_far)
        page_json['id'] += '&iris=1'
        if 'prev' in page_json:
            page_json['prev'] += '&iris=1'
        if 'next' in page_json:
            page_json['next'] += '&iris=1'
    else:
        page_json['items'] = annotations(so_far)

    return dump_json(page_json)


@wptserve.handlers.handler
def annotation_get(request, response):
    """Individual Annotations"""
    requested_file = doc_root + request.request_path[1:]
    base = os.path.basename( requested_file )

    headers_file = doc_root + 'annotations/annotation.headers'

    if base.startswith("temp-") and tempAnnotations[base]:
        response.headers.update(load_headers_from_file(headers_file))
        response.headers.set('Etag', hashlib.sha1(base).hexdigest())
        data = dump_json(tempAnnotations[base])
        if data != "" :
            response.content = data
            response.status = 200
        else:
            response.content = ""
            response.status = 404
    elif os.path.isfile(requested_file):
        response.headers.update(load_headers_from_file(headers_file))
        # Calculate ETag using Apache httpd's default method (more or less)
        # http://www.askapache.info//2.3/mod/core.html#fileetag
        statinfo = os.stat(requested_file)
        etag = "{0}{1}{2}".format(statinfo.st_ino, statinfo.st_mtime,
                                  statinfo.st_size)
        # obfuscate so we don't leak info; hexdigest for string compatibility
        response.headers.set('Etag', hashlib.sha1(etag).hexdigest())

        with open(requested_file, 'r') as data_file:
            data = data_file.read()
        response.content = data
        response.status = 200
    else:
        response.content = 'Not Found'
        response.status = 404

    add_cors_headers(response)


@wptserve.handlers.handler
def annotation_head(request, response):
    requested_file = doc_root + request.request_path[1:]
    base = os.path.basename(requested_file)

    headers_file = doc_root + 'annotations/annotation.options.headers'

    if base.startswith("temp-") and tempAnnotations[base]:
        response.status = 200
        response.headers.update(load_headers_from_file(headers_file))
    elif os.path.isfile(requested_file):
        response.status = 200
        response.headers.update(load_headers_from_file(headers_file))
    else:
        response.status = 404

    add_cors_headers(response)

@wptserve.handlers.handler
def annotation_options(request, response):
    requested_file = doc_root + request.request_path[1:]
    base = os.path.basename(requested_file)

    headers_file = doc_root + 'annotations/annotation.options.headers'

    if base.startswith("temp-") and tempAnnotations[base]:
        response.status = 200
        response.headers.update(load_headers_from_file(headers_file))
    elif os.path.isfile(requested_file):
        response.status = 200
        response.headers.update(load_headers_from_file(headers_file))
    else:
        response.status = 404

    add_cors_headers(response)

def create_annotation(body):
    # TODO: verify media type is JSON of some kind (at least)
    incoming = json.loads(body)
    id = "temp-"+str(uuid.uuid4())
    if 'id' in incoming:
        incoming['canonical'] = incoming['id']
    incoming['id'] = URIroot + '/annotations/' + id

    return incoming


@wptserve.handlers.handler
def annotation_post(request, response):
    incoming = create_annotation(request.body)
    newID = incoming['id']
    key = os.path.basename(newID)

    print("post:" + newID)
    print("post:" + key)

    tempAnnotations[key] = dump_json(incoming)

    headers_file = doc_root + 'annotations/annotation.headers'
    response.headers.update(load_headers_from_file(headers_file))
    response.headers.append('Location', newID)
    add_cors_headers(response)
    response.content = dump_json(incoming)
    response.status = 201

@wptserve.handlers.handler
def annotation_put(request, response):
    incoming = create_annotation(request.body)

    # remember it in our local cache too
    # tempAnnotations[request.request_path[1:]] = dump_jason(incoming)
    newID = incoming['id']
    key = os.path.basename(newID)

    print("put:" + newID)
    print("put:" + key)

    tempAnnotations[key] = dump_json(incoming)

    headers_file = doc_root + 'annotations/annotation.headers'
    response.headers.update(load_headers_from_file(headers_file))
    response.headers.append('Location', incoming['id'])
    add_cors_headers(response)
    response.content = dump_json(incoming)
    response.status = 200


@wptserve.handlers.handler
def annotation_delete(request, response):
    base = os.path.basename(request.request_path[1:])
    requested_file = doc_root + request.request_path[1:]

    add_cors_headers(response)

    headers_file = doc_root + 'annotations/annotation.headers'

    try:
        if base.startswith("temp-"):
            if tempAnnotations[base]:
                del tempAnnotations[base]
        else:
            os.remove(requested_file)
        response.headers.update(load_headers_from_file(headers_file))
        response.status = 204
        response.content = ''
    except OSError:
        response.status = 404
        response.content = 'Not Found'

if __name__ == '__main__':
    print('http://' + myhost + ':{0}/'.format(port))
    print('container URI is http://' + myhost + ':{0}/'.format(port) + "/annotations/")
    print('example annotation URI is http://' + myhost + ':{0}/'.format(port) + "/annotations/anno1.json")

    routes = [
        ("GET", "", wptserve.handlers.file_handler),
        ("GET", "index.html", wptserve.handlers.file_handler),

        # container/collection responses
        ("HEAD", "annotations/", collection_head),
        ("OPTIONS", "annotations/", collection_options),
        ("GET", "annotations/", collection_get),

        # create annotations in the collection
        ("POST", "annotations/", annotation_post),

        # single annotation responses
        ("HEAD", "annotations/*", annotation_head),
        ("OPTIONS", "annotations/*", annotation_options),
        ("GET", "annotations/*", annotation_get),
        ("PUT", "annotations/*", annotation_put),
        ("DELETE", "annotations/*", annotation_delete)
    ]

    httpd = wptserve.server.WebTestHttpd(host=myhost, bind_hostname=myhost, port=port, doc_root=doc_root,
                                         routes=routes)
    httpd.start(block=True)
