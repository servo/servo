import os
import sys
import unittest

sys.path.insert(1, os.path.abspath(os.path.join(__file__, "../..")))
import base_test


class EcmasScriptTest(base_test.WebDriverBaseTest):
    def test_that_ecmascript_returns_document_title(self):
        self.driver.get(self.webserver.where_is("ecmascript/res/ecmascript_test.html"))
        result = self.driver.execute_script("return document.title;");
        self.assertEquals("ecmascript test", result);


if __name__ == "__main__":
    unittest.main()
