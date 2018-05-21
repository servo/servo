# Copyright 2014 Google Inc. All rights reserved.
#
# Use of this source code is governed by a BSD-style
# license that can be found in the COPYING file or at
# https://developers.google.com/open-source/licenses/bsd


import urlparse


def _add_set_cookie(request, value):
    request.extra_headers.append(('Set-Cookie', value))


def web_socket_do_extra_handshake(request):
    components = urlparse.urlparse(request.uri)
    command = components[4]

    ONE_DAY_LIFE = 'Max-Age=86400'

    if command == 'set':
        _add_set_cookie(request, '; '.join(['foo=bar', ONE_DAY_LIFE]))
    elif command == 'set_httponly':
        _add_set_cookie(request,
            '; '.join(['httpOnlyFoo=bar', ONE_DAY_LIFE, 'httpOnly']))
    elif command == 'clear':
        _add_set_cookie(request, 'foo=0; Max-Age=0')
        _add_set_cookie(request, 'httpOnlyFoo=0; Max-Age=0')


def web_socket_transfer_data(request):
    pass
