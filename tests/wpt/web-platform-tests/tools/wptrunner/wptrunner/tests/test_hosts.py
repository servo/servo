import unittest
import sys
from os.path import join, dirname
from cStringIO import StringIO

sys.path.insert(0, join(dirname(__file__), "..", ".."))

from wptrunner import hosts


class HostsTest(unittest.TestCase):
    def do_test(self, input, expected):
        host_file = hosts.HostsFile.from_file(StringIO(input))
        self.assertEquals(host_file.to_string(), expected)

    def test_simple(self):
        self.do_test("""127.0.0.1    \tlocalhost  alias # comment
# Another comment""",
                     """127.0.0.1 localhost alias # comment
# Another comment
""")

    def test_blank_lines(self):
        self.do_test("""127.0.0.1    \tlocalhost  alias # comment

\r
    \t
# Another comment""",
                     """127.0.0.1 localhost alias # comment
# Another comment
""")

    def test_whitespace(self):
        self.do_test("""    \t127.0.0.1    \tlocalhost  alias # comment     \r
    \t# Another comment""",
                     """127.0.0.1 localhost alias # comment
# Another comment
""")

    def test_alignment(self):
        self.do_test("""127.0.0.1    \tlocalhost  alias
192.168.1.1 another_host    another_alias
""","""127.0.0.1   localhost    alias
192.168.1.1 another_host another_alias
"""
)

    def test_multiple_same_name(self):
        # The semantics are that we overwrite earlier entries with the same name
        self.do_test("""127.0.0.1    \tlocalhost  alias
192.168.1.1 localhost    another_alias""","""192.168.1.1 localhost another_alias
"""
)

if __name__ == "__main__":
    unittest.main()
