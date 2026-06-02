import sys
import unittest

from mozlog import capture, structuredlog
from test_structured import LogHandler


class TestCaptureIO(unittest.TestCase):
    """Tests expected logging output of CaptureIO"""

    def setUp(self):
        self.logger = structuredlog.StructuredLogger("test")
        self.handler = LogHandler()
        self.logger.add_handler(self.handler)

    def test_captureio_log(self):
        """
        CaptureIO takes in two arguments. The second argument must
        be truthy in order for the code to run. Hence, the string
        "capture_stdio" has been used in this test case.
        """
        with capture.CaptureIO(self.logger, "capture_stdio"):
            print("message 1")
            sys.stdout.write("message 2")
            sys.stderr.write("message 3")
            sys.stdout.write("\xff")
        log = self.handler.items
        messages = [item["message"] for item in log]
        self.assertIn("STDOUT: message 1", messages)
        self.assertIn("STDOUT: message 2", messages)
        self.assertIn("STDERR: message 3", messages)
        self.assertIn(u"STDOUT: \xff", messages)


if __name__ == "__main__":
    import mozunit
    mozunit.main()
