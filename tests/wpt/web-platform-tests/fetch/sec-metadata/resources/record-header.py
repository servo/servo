import os
import uuid
import hashlib
import time

def main(request, response):
  ## Get the query parameter (key) from URL ##
  ## Tests will record POST requests (CSP Report) and GET (rest) ##
  if request.GET:
    key = request.GET['file']
  elif request.POST:
    key = request.POST['file']

  ## Convert the key from String to UUID valid String ##
  testId = hashlib.md5(key).hexdigest()

  ## Handle the header retrieval request ##
  if 'retrieve' in request.GET:
    response.writer.write_status(200)
    response.writer.end_headers()
    try:
      header_value = request.server.stash.take(testId)
      response.writer.write(header_value)
    except (KeyError, ValueError) as e:
      response.writer.write("No header has been recorded")
      pass

    response.close_connection = True

  ## Record incoming Sec-Metadata header value
  else:
    try:
      ## Return empty string as a default value ##
      header = request.headers.get("Sec-Metadata", "")
      request.server.stash.put(testId, header)
    except KeyError:
      ## The header is already recorded or it doesn't exist
      pass

    ## Prevent the browser from caching returned responses and allow CORS ##
    response.headers.set("Access-Control-Allow-Origin", "*")
    response.headers.set("Cache-Control", "no-cache, no-store, must-revalidate")
    response.headers.set("Pragma", "no-cache")
    response.headers.set("Expires", "0")

    ## Add a valid ServiceWorker Content-Type ##
    if key.startswith("serviceworker"):
      response.headers.set("Content-Type", "application/javascript")

    ## Return a valid .vtt content for the <track> tag ##
    if key.startswith("track"):
      return "WEBVTT"

    ## Return a valid SharedWorker ##
    if key.startswith("sharedworker"):
      response.headers.set("Content-Type", "application/javascript")
      file = open(os.path.join(request.doc_root, "fetch", "sec-metadata",
                               "resources", "sharedWorker.js"), "r")
      shared_worker = file.read()
      file.close()
      return shared_worker

    ## Return a valid font content and Content-Type ##
    if key.startswith("font"):
      response.headers.set("Content-Type", "application/x-font-ttf")
      file = open(os.path.join(request.doc_root, "fonts", "Ahem.ttf"), "r")
      font = file.read()
      file.close()
      return font

    ## Return a valid audio content and Content-Type ##
    if key.startswith("audio"):
      response.headers.set("Content-Type", "audio/mpeg")
      file = open(os.path.join(request.doc_root, "media", "sound_5.mp3"), "r")
      audio = file.read()
      file.close()
      return audio

    ## Return a valid video content and Content-Type ##
    if key.startswith("video"):
      response.headers.set("Content-Type", "video/mp4")
      file = open(os.path.join(request.doc_root, "media", "A4.mp4"), "r")
      video = file.read()
      file.close()
      return video

    ## Return a valid style content and Content-Type ##
    if key.startswith("style") or key.startswith("embed") or key.startswith("object"):
      response.headers.set("Content-Type", "text/html")
      return "<html>EMBED!</html>"

    ## Return a valid image content and Content-Type for redirect requests ##
    if key.startswith("redirect"):
      response.headers.set("Content-Type", "image/jpeg")
      file = open(os.path.join(request.doc_root, "media", "1x1-green.png"), "r")
      image = file.read()
      file.close()
      return image

    ## Return a valid dedicated worker
    if key.startswith("worker"):
      response.headers.set("Content-Type", "application/javascript")
      return "self.postMessage('loaded');"

    ## Return a valid XSLT
    if key.startswith("xslt"):
      response.headers.set("Content-Type", "text/xsl")
      return """<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
  <xsl:template match="@*|node()">
    <xsl:copy>
      <xsl:apply-templates select="@*|node()"/>
    </xsl:copy>
  </xsl:template>
</xsl:stylesheet>"""

