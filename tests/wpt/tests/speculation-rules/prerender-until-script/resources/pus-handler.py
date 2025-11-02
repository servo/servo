"""
This script handles requests for the "main" and "signal" resources.
The "main" resource serves an HTML page that either initiates or acts as the
target for a prerendering speculation rule. The "signal" resource is used to
coordinate between the initiator and the prerendered page, ensuring the
prerendering is complete before proceeding. It uses query parameters to
determine its behavior and coordinate the process.
"""

import os
import copy
import time
from urllib.parse import parse_qs

def encode_query(query_dict):
    encoded_pairs = [f"{key}={value[0]}" if value[0] !="" else key for key, value in query_dict.items()]
    return '&'.join(encoded_pairs)

def calculate_signal_path(url_parts):
    url_parts = copy.deepcopy(url_parts)
    query_dict = parse_qs(url_parts.query, keep_blank_values=True)
    query_dict['type'] = ['signal']
    return url_parts._replace(query=encode_query(query_dict)).geturl()

def calculate_prerendering_path(url_parts):
    query_dict = parse_qs(url_parts.query)
    query_dict['isprerendering'] = [True]
    return url_parts._replace(query=encode_query(query_dict)).geturl()

def main_resource_handler(request, response):
    is_prerendering = b"isprerendering" in request.GET
    uid = request.GET.get(b"uid")
    signal_path =  calculate_signal_path(request.url_parts)
    content = ''
    if not is_prerendering:
        # First request, which is the initiator
        template_path = os.path.join(os.path.dirname(__file__), "pus-initiator-page-template.html")
        prerendering_path = calculate_prerendering_path(request.url_parts)
        with open(template_path, "r") as f:
          content = f.read().replace("{{signal_url}}", signal_path).replace(
            "{{prerendering_url}}", prerendering_path)
    else:
        template_path = os.path.join(os.path.dirname(
           __file__), "pus-page-template.html")
        with open(template_path, "r") as f:
           content = f.read().replace("{{signal_path}}", signal_path)
    response.headers.set(b"Content-Type", b"text/html")
    response.status = 200
    response.content = content.encode('utf-8')

def signal_handler(request, response):
    is_prerendering = b"isprerendering" in request.GET
    uid = request.GET.get(b"uid")
    if is_prerendering:
      with request.server.stash.lock:
        request.server.stash.put(uid, "ok")
    else:
      # This will hang until the gate is released
      while True:
        with request.server.stash.lock:
          if request.server.stash.take(uid) is None:
            time.sleep(0.1)
          else:
             break
    response.headers.set(b"Content-Type", b"text/javascript")
    response.status = 200
    response.content = "console.error('orz')".encode('utf-8')

def main(request, response):
    resource_router  = {
        b"main":main_resource_handler,
        b"signal": signal_handler
    }
    resource_type = request.GET.get(b"type")
    resource_handler = resource_router.get(resource_type)
    if resource_handler is not None:
       return resource_handler(request, response)
    response.status = 400  # Bad Request
    response.content = b"Invalid resource type"
    return
