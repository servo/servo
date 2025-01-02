import os
import progressbar
from urllib.request import urlopen

UnicodeXMLURL = "https://raw.githubusercontent.com/w3c/xml-entities/gh-pages/unicode.xml"
InlineAxisOperatorsURL = "https://w3c.github.io/mathml-core/tables/inline-axis-operators.txt"


def downloadWithProgressBar(url, outputDirectory="./", forceDownload=False):

    baseName = os.path.basename(url)
    fileName = os.path.join(outputDirectory, baseName)

    if not forceDownload and os.path.exists(fileName):
        return fileName

    request = urlopen(url)
    totalSize = int(request.info().get('Content-Length').strip())
    bar = progressbar.ProgressBar(maxval=totalSize).start()

    chunkSize = 16 * 1024
    downloaded = 0
    print("Downloading %s" % url)
    os.umask(0o002)
    with open(fileName, 'wb') as fp:
        while True:
            chunk = request.read(chunkSize)
            downloaded += len(chunk)
            bar.update(downloaded)
            if not chunk:
                break
            fp.write(chunk)
        bar.finish()

    return fileName
