import sys
import py
from py._path import svnwc as svncommon
from common import CommonFSTests

class CommonSvnTests(CommonFSTests):

    def test_propget(self, path1):
        url = path1.join("samplefile")
        value = url.propget('svn:eol-style')
        assert value == 'native'

    def test_proplist(self, path1):
        url = path1.join("samplefile")
        res = url.proplist()
        assert res['svn:eol-style'] == 'native'

    def test_info(self, path1):
        url = path1.join("samplefile")
        res = url.info()
        assert res.size > len("samplefile") and res.created_rev >= 0

    def test_log_simple(self, path1):
        url = path1.join("samplefile")
        logentries = url.log()
        for logentry in logentries:
            assert logentry.rev == 1
            assert hasattr(logentry, 'author')
            assert hasattr(logentry, 'date')

#cache.repositories.put(svnrepourl, 1200, 0)
