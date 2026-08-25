import os
import re

from wptserve.utils import isomorphic_decode, isomorphic_encode

def main(request, response):
    type = request.GET[b'type']
    encoding = request.GET[b'encoding']
    # We want the raw input for 'q'
    q = re.search(r'q=([^&]+)', request.url_parts.query).groups()[0]
    if type == b'html':
        return [(b"Content-Type", b"text/html; charset=utf-8")], isomorphic_encode(q)
    elif type == b'css':
        return [(b"Content-Type", b"text/css; charset=utf-8")], b"#test::before { content:'%s' }" % isomorphic_encode(q)
    elif type == b'js':
        return [(b"Content-Type", b"text/javascript; charset=utf-8")], b"%s = '%s';" % (request.GET[b'var'], isomorphic_encode(q))
    elif type == b'worker':
        return [(b"Content-Type", b"text/javascript")], b"postMessage('%s'); close();" % isomorphic_encode(q)
    elif type == b'sharedworker':
        return [(b"Content-Type", b"text/javascript")], b"onconnect = function(e) { e.source.postMessage('%s'); close(); };" % isomorphic_encode(q)
    elif type == b'worker_importScripts':
        return ([(b"Content-Type", b"text/javascript; charset=%s" % encoding)], # charset should be ignored for workers
                b"""try {
                     var x = 'importScripts failed to run';
                     importScripts('?q=\\u00E5&type=js&var=x&encoding=%s');
                     postMessage(x);
                     close();
                   } catch(ex) {
                     postMessage(String(ex));
                   }""" % encoding)
    elif type == b'worker_worker':
        return ([(b"Content-Type", b"text/javascript; charset=%s" % encoding)], # charset should be ignored for workers
                b"""try {
                     var worker = new Worker('?q=\\u00E5&type=worker&encoding=%s');
                     worker.onmessage = function(e) {
                       postMessage(e.data);
                       close();
                     };
                   } catch(ex) {
                     postMessage(String(ex));
                   }""" % encoding)
    elif type == b'sharedworker_importScripts':
        return ([(b"Content-Type", b"text/javascript; charset=%s" % encoding)], # charset should be ignored for workers
                b"""var x = 'importScripts failed to run';
                     onconnect = function(e) {
                     var connect_port = e.source;
                     try {
                       importScripts('?q=\\u00E5&type=js&var=x&encoding=%s');
                       connect_port.postMessage(x);
                       close();
                     } catch(ex) {
                       connect_port.postMessage(String(ex));
                     }
                   };""" % encoding)
    elif type == b'sharedworker_worker':
        return ([(b"Content-Type", b"text/javascript; charset=%s" % encoding)], # charset should be ignored for workers
                b"""onconnect = function(e) {
                     var connect_port = e.source;
                     try {
                       var worker = new Worker('?q=\\u00E5&type=worker&encoding=%s');
                       worker.onmessage = function(e) {
                         connect_port.postMessage(e.data);
                         close();
                       };
                     } catch(ex) {
                       connect_port.postMessage(String(ex));
                     }
                   };""" % encoding)
    elif type == b'eventstream':
        return [(b"Content-Type", b"text/event-stream")], b"data: %s\n\n" % isomorphic_encode(q)
    elif type == b'svg':
        return [(b"Content-Type", b"image/svg+xml")], b"<svg xmlns='http://www.w3.org/2000/svg'>%s</svg>" % isomorphic_encode(q)
    elif type == b'xmlstylesheet_css':
        return ([(b"Content-Type", b"application/xhtml+xml; charset=%s" % encoding)],
                (u"""<?xml-stylesheet href="?q=&#x00E5;&amp;type=css&amp;encoding=%s"?><html xmlns="http://www.w3.org/1999/xhtml"/>""" % isomorphic_decode(encoding))
                .encode(isomorphic_decode(encoding)))
    elif type == b'png':
        if q == u'%E5' or q == u'%26%23229%3B':
            image = u'green-1x1.png'
        elif q == u'%C3%A5':
            image = u'green-2x2.png'
        elif q == u'%3F':
            image = u'green-16x16.png'
        else:
            image = u'green-256x256.png'
        rv = open(os.path.join(request.doc_root, u"images", image), "rb").read()
        return [(b"Content-Type", b"image/png")], rv
    elif type == b'video':
        ext = request.GET[b'ext']
        if q == u'%E5' or q == u'%26%23229%3B':
            video = u'A4' # duration: 3
        elif q == u'%C3%A5':
            video = u'movie_5' # duration: 5
        elif q == u'%3F':
            video = u'green-at-15' # duration: 30
        else:
            video = u'movie_300' # duration: 300
        rv = open(os.path.join(request.doc_root, u"media", u"%s.%s" % (video, isomorphic_decode(ext))), "rb").read()
        if ext == b'ogv':
            ext = b'ogg'
        return [(b"Content-Type", b"video/%s" % ext)], rv
    elif type == b'webvtt':
        return [(b"Content-Type", b"text/vtt")], b"WEBVTT\n\n00:00:00.000 --> 00:00:01.000\n%s" % isomorphic_encode(q)
