import os
import subprocess
import time
import sys
import urllib2


class WPTServer(object):
    def __init__(self, wpt_root):
        self.wpt_root = wpt_root
        sys.path.insert(0, os.path.join(wpt_root, "tools"))
        from serve.serve import Config
        config = Config()
        self.host = config["browser_host"]
        self.http_port = config["ports"]["http"][0]
        self.https_port = config["ports"]["https"][0]
        self.base_url = 'http://%s:%s' % (self.host, self.http_port)
        self.https_base_url = 'https://%s:%s' % (self.host, self.https_port)

    def start(self):
        self.devnull = open(os.devnull, 'w')
        self.proc = subprocess.Popen(
            [os.path.join(self.wpt_root, 'wpt'), 'serve'],
            stderr=self.devnull,
            cwd=self.wpt_root)

        for retry in range(5):
            # Exponential backoff.
            time.sleep(2 ** retry)
            if self.proc.poll() != None:
                break
            try:
                urllib2.urlopen(self.base_url, timeout=1)
                return
            except urllib2.URLError:
                pass

        raise Exception('Could not start wptserve.')

    def stop(self):
        self.proc.terminate()
        self.proc.wait()
        self.devnull.close()

    def url(self, abs_path):
        return self.https_base_url + '/' + os.path.relpath(abs_path, self.wpt_root)
