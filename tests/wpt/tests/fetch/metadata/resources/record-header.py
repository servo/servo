import os
import hashlib
import json

from wptserve.utils import isomorphic_decode

def main(request, response):
  ## Get the query parameter (key) from URL ##
  ## Tests will record POST requests (CSP Report) and GET (rest) ##
  if request.GET:
    key = request.GET[b'file']
  elif request.POST:
    key = request.POST[b'file']

  ## Convert the key from String to UUID valid String ##
  testId = hashlib.md5(key).hexdigest()

  ## Handle the header retrieval request ##
  if b'retrieve' in request.GET:
    response.writer.write_status(200)
    response.writer.write_header(b"Connection", b"close")
    response.writer.end_headers()
    try:
      header_value = request.server.stash.take(testId)
      response.writer.write(header_value)
    except (KeyError, ValueError) as e:
      response.writer.write(u"No header has been recorded")
      pass

    response.close_connection = True

  ## Record incoming fetch metadata header value
  else:
    try:
      ## Return a serialized JSON object with one member per header. If the ##
      ## header isn't present, the member will contain an empty string.     ##
      header = json.dumps({
        u"dest": isomorphic_decode(request.headers.get(b"sec-fetch-dest", b"")),
        u"mode": isomorphic_decode(request.headers.get(b"sec-fetch-mode", b"")),
        u"site": isomorphic_decode(request.headers.get(b"sec-fetch-site", b"")),
        u"user": isomorphic_decode(request.headers.get(b"sec-fetch-user", b"")),
      })
      request.server.stash.put(testId, header)
    except KeyError:
      ## The header is already recorded or it doesn't exist
      pass

    ## Prevent the browser from caching returned responses and allow CORS ##
    response.headers.set(b"Access-Control-Allow-Origin", b"*")
    response.headers.set(b"Cache-Control", b"no-cache, no-store, must-revalidate")
    response.headers.set(b"Pragma", b"no-cache")
    response.headers.set(b"Expires", b"0")

    ## Add a valid ServiceWorker Content-Type ##
    if key.startswith(b"serviceworker"):
      response.headers.set(b"Content-Type", b"application/javascript")

    ## Add a valid image Content-Type ##
    if key.startswith(b"image"):
      response.headers.set(b"Content-Type", b"image/png")
      file = open(os.path.join(request.doc_root, u"media", u"1x1-green.png"), u"rb")
      image = file.read()
      file.close()
      return image

    ## Return a valid .vtt content for the <track> tag ##
    if key.startswith(b"track"):
      return b"WEBVTT"

    ## Return a valid SharedWorker ##
    if key.startswith(b"sharedworker"):
      response.headers.set(b"Content-Type", b"application/javascript")
      file = open(os.path.join(request.doc_root, u"fetch", u"metadata",
                               u"resources", u"sharedWorker.js"), u"rb")
      shared_worker = file.read()
      file.close()
      return shared_worker

    ## Return a valid font content and Content-Type ##
    if key.startswith(b"font"):
      response.headers.set(b"Content-Type", b"application/x-font-ttf")
      file = open(os.path.join(request.doc_root, u"fonts", u"Ahem.ttf"), u"rb")
      font = file.read()
      file.close()
      return font

    ## Return a valid audio content and Content-Type ##
    if key.startswith(b"audio"):
      response.headers.set(b"Content-Type", b"audio/mpeg")
      file = open(os.path.join(request.doc_root, u"media", u"sound_5.mp3"), u"rb")
      audio = file.read()
      file.close()
      return audio

    ## Return a valid video content and Content-Type ##
    if key.startswith(b"video"):
      response.headers.set(b"Content-Type", b"video/mp4")
      file = open(os.path.join(request.doc_root, u"media", u"A4.mp4"), u"rb")
      video = file.read()
      file.close()
      return video

    ## Return valid style content and Content-Type ##
    if key.startswith(b"style"):
      response.headers.set(b"Content-Type", b"text/css")
      return b"div { }"

    ## Return a valid embed/object content and Content-Type ##
    if key.startswith(b"embed") or key.startswith(b"object"):
      response.headers.set(b"Content-Type", b"text/html")
      return b"<html>EMBED!</html>"

    ## Return a valid image content and Content-Type for redirect requests ##
    if key.startswith(b"redirect"):
      response.headers.set(b"Content-Type", b"image/jpeg")
      file = open(os.path.join(request.doc_root, u"media", u"1x1-green.png"), u"rb")
      image = file.read()
      file.close()
      return image

    ## Return a valid dedicated worker
    if key.startswith(b"worker"):
      response.headers.set(b"Content-Type", b"application/javascript")
      return b"self.postMessage('loaded');"

    ## Return a valid worklet
    if key.startswith(b"worklet"):
      response.headers.set(b"Content-Type", b"application/javascript")
      return b""

    ## Return a valid XSLT
    if key.startswith(b"xslt"):
      response.headers.set(b"Content-Type", b"text/xsl")
      return b"""<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:template match="@*|node()">
    <xsl:copy>
      <xsl:apply-templates select="@*|node()"/>
    </xsl:copy>
  </xsl:template>
</xsl:stylesheet>"""

    if key.startswith(b"script"):
      response.headers.set(b"Content-Type", b"application/javascript")
      return b"void 0;"
