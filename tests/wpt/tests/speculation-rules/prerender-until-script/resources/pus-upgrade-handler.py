"""
Serves the initiator and target pages for the PUS-to-prerender upgrade test.

Two stash-backed gates sequence the test:
  * `uid` confirms the PUS target started loading before the upgrade rule.
  * `js_uid` confirms its paused JavaScript resumed before activation.

Prerender-page requests release a gate; initiator requests long-poll it.
"""

import os
import time
from urllib.parse import parse_qs


def _encode_query(query_dict):
    encoded_pairs = [
        f"{key}={value[0]}" if value[0] != "" else key
        for key, value in query_dict.items()
    ]
    return '&'.join(encoded_pairs)


def _calculate_signal_path(url_parts, phase):
    query_dict = parse_qs(url_parts.query, keep_blank_values=True)
    query_dict['type'] = ['signal']
    query_dict['phase'] = [phase]
    return url_parts._replace(query=_encode_query(query_dict)).geturl()


def _calculate_prerendering_path(url_parts):
    query_dict = parse_qs(url_parts.query, keep_blank_values=True)
    # Presence-only flag; `_encode_query` emits it bare when the value is empty.
    query_dict['isprerendering'] = ['']
    return url_parts._replace(query=_encode_query(query_dict)).geturl()


def _stash_key_for_phase(request, phase):
    if phase == 'pus-loaded':
        return request.GET.get(b"uid")
    if phase == 'js-ran':
        return request.GET.get(b"js_uid")
    return None


def _main_resource_handler(request, response):
    is_prerendering = b"isprerendering" in request.GET
    pus_loaded_path = _calculate_signal_path(request.url_parts, 'pus-loaded')
    js_ran_path = _calculate_signal_path(request.url_parts, 'js-ran')
    if not is_prerendering:
        template_name = 'pus-upgrade-initiator-template.html'
        prerendering_path = _calculate_prerendering_path(request.url_parts)
        substitutions = {
            '{{pus_loaded_url}}': pus_loaded_path,
            '{{js_ran_url}}': js_ran_path,
            '{{prerendering_url}}': prerendering_path,
        }
    else:
        template_name = 'pus-upgrade-page-template.html'
        substitutions = {
            '{{pus_loaded_path}}': pus_loaded_path,
            '{{js_ran_path}}': js_ran_path,
        }
    template_path = os.path.join(os.path.dirname(__file__), template_name)
    with open(template_path, 'r') as f:
        content = f.read()
    for placeholder, value in substitutions.items():
        content = content.replace(placeholder, value)
    response.headers.set(b"Content-Type", b"text/html")
    response.status = 200
    response.content = content.encode('utf-8')


def _signal_handler(request, response):
    is_prerendering = b"isprerendering" in request.GET
    phase = request.GET.get(b"phase", b"").decode('utf-8')
    key = _stash_key_for_phase(request, phase)
    if key is None:
        response.status = 400
        response.content = b"Invalid or missing phase"
        return
    if is_prerendering:
        # Make retried release requests idempotent; `stash.put` rejects an
        # existing key.
        with request.server.stash.lock:
            request.server.stash.take(key)
            request.server.stash.put(key, "ok")
    else:
        # Bound the wait so a failed upgrade cannot leak the worker and popup.
        # This exceeds the test's 30s timeout so its diagnostic remains primary;
        # the 504 only unblocks the initiator for cleanup.
        for _ in range(600):
            with request.server.stash.lock:
                if request.server.stash.take(key) is not None:
                    break
            time.sleep(0.1)
        else:
            response.headers.set(b"Content-Type", b"text/plain")
            response.status = 504
            response.content = (
                b"Gate not released within 60s; "
                b"prerender-until-script upgrade likely did not occur")
            return
    response.headers.set(b"Content-Type", b"text/javascript")
    response.status = 200
    response.content = b"// signal acknowledged"


def main(request, response):
    resource_router = {
        b"main": _main_resource_handler,
        b"signal": _signal_handler,
    }
    resource_type = request.GET.get(b"type")
    resource_handler = resource_router.get(resource_type)
    if resource_handler is not None:
        return resource_handler(request, response)
    response.status = 400
    response.content = b"Invalid resource type"
