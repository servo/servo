from os import path
from os import listdir

"""
The main purpose of this script is to set cookies based on files in this folder:
    cookies/http-state/resources

If a wpt server is running, navigate to
    http://<server>/cookies/http-state/resources/cookie-setter.py
which will run all cookie tests and explain the usage of this script in detail.

If you want to run a test in isolation, append "?debug=" and the test id to the
URL above.
"""

SETUP_FILE_TEMPLATE = "{}-test"
EXPECTATION_FILE_TEMPLATE = "{}-expected"
EXPECTATION_HTML_SCAFFOLD = "iframe-expectation-doc.html.py-str"
DEBUGGING_HTML_SCAFFOLD = "debugging-single-test.html.py-str"
DEFAULT_RESOURCE_DIR = path.join("cookies", "http-state", "resources")
DEFAULT_TEST_DIR = "test-files"
ALL_TESTS = "all-tests.html.py-str"


def dump_file(directory, filename):
  """Reads a file into a string."""
  return open(path.join(directory, filename), "r").read()


class CookieTestResponse(object):
  """Loads the Set-Cookie header from a given file name. Relies on the naming
  convention that ever test file is called '<test_id>-test' and every
  expectation is called '<test_id>-expected'."""
  def __init__(self, file, root):
    super(CookieTestResponse, self).__init__()
    self.__test_file = SETUP_FILE_TEMPLATE.format(file)
    self.__expectation_file = EXPECTATION_FILE_TEMPLATE.format(file)
    self.__resources_dir = path.join(root, DEFAULT_RESOURCE_DIR)
    self.__test_files_dir = path.join(self.__resources_dir, DEFAULT_TEST_DIR)

  def cookie_setting_header(self):
    """Returns the loaded header."""
    return dump_file(self.__test_files_dir, self.__test_file)

  def body_with_expectation(self):
    """Returns a full HTML document which contains the expectation."""
    html_frame = dump_file(self.__resources_dir, EXPECTATION_HTML_SCAFFOLD)
    expected_data = dump_file(self.__test_files_dir, self.__expectation_file)
    return html_frame.format(**{'data' : expected_data})

def find_all_test_files(root):
  """Retrieves all files from the test-files/ folder and returns them as
  string pair as used in the JavaScript object. Sorted by filename."""
  all_files = []
  line_template = '{{file: "{filename}", name: "{filename}"}},'
  for file in listdir(path.join(root, DEFAULT_RESOURCE_DIR, DEFAULT_TEST_DIR)):
    if file.endswith('-test'):
      name = file.replace('-test', '')
      all_files.append(line_template.format(**{'filename' : name}))
  all_files.sort()
  return all_files

def generate_for_all_tests(root):
  """Returns an HTML document which loads and executes all tests in the
  test-files/ directory."""
  html_frame = dump_file(path.join(root, DEFAULT_RESOURCE_DIR), ALL_TESTS)
  return html_frame % '\n'.join(find_all_test_files(root))

def main(request, response):
  if "debug" in request.GET:
    """If '?debug=' is set, return the document for a single test."""
    response.writer.write_status(200)
    response.writer.end_headers()
    html_frame = dump_file(path.join(request.doc_root, DEFAULT_RESOURCE_DIR),
                           DEBUGGING_HTML_SCAFFOLD)
    test_file = html_frame % (request.GET['debug'])
    response.writer.write_content(test_file)
    return;

  if "file" in request.GET:
    """If '?file=' is set, send a cookie and a document which contains the
    expectation of which cookies should be set by the browser in response."""
    cookie_response = CookieTestResponse(request.GET['file'], request.doc_root)

    response.writer.write_status(200)
    response.writer.write(cookie_response.cookie_setting_header())
    response.writer.end_headers()
    response.writer.write_content(cookie_response.body_with_expectation())
    return;

  """Without any arguments, return documentation and run all available tests."""
  response.writer.write_status(200)
  response.writer.end_headers()
  response.writer.write_content(generate_for_all_tests(request.doc_root))
