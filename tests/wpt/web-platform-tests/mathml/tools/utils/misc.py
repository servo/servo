from __future__ import print_function
import os
import progressbar
import urllib2

MathMLAssociationCopyright = "Copyright (c) 2016 MathML Association"

def downloadWithProgressBar(url, outputDirectory="./", forceDownload=False):

    baseName = os.path.basename(url)
    fileName = os.path.join(outputDirectory, baseName)

    if not forceDownload and os.path.exists(fileName):
        return fileName

    request = urllib2.urlopen(url)
    totalSize = int(request.info().getheader('Content-Length').strip())
    bar = progressbar.ProgressBar(maxval=totalSize).start()

    chunkSize = 16 * 1024
    downloaded = 0
    print("Downloading %s" % url)
    os.umask(0002)
    with open(fileName, 'wb') as fp:
        while True:
            chunk = request.read(chunkSize)
            downloaded += len(chunk)
            bar.update(downloaded)
            if not chunk: break
            fp.write(chunk)
        bar.finish()

    return fileName
