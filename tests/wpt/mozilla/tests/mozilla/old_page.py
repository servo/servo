# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

def main(request, response):
    response.headers.set(b"Age", b"90000")
    response.headers.set(b"Last-Modified", b"Wed, 21 Oct 2015 07:28:00 GMT")
    response.write_status_headers()
    response.writer.write_content(b"Body")
