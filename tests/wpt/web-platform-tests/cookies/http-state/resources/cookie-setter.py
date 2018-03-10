from os import path;


SETUP_FILE_TEMPLATE = "{}-test"
EXPECTATION_FILE_TEMPLATE = "{}-expected"
EXPECTATION_HTML_SCAFFOLD = "iframe-expectation-doc.html.py-str"
DEBUGGING_HTML_SCAFFOLD = "debugging-single-test.html.py-str"
DEFAULT_RESOURCE_DIR = path.join("cookies", "http-state", "resources")
DEFAULT_TEST_DIR = "test-files"


def dump_file(directory, filename):
  return open(path.join(directory, filename), "r").read()


class CookieTestResponse(object):
  def __init__(self, file, root):
    super(CookieTestResponse, self).__init__()
    self.__test_file = SETUP_FILE_TEMPLATE.format(file)
    self.__expectation_file = EXPECTATION_FILE_TEMPLATE.format(file)
    self.__resources_dir = path.join(root, DEFAULT_RESOURCE_DIR)
    self.__test_files_dir = path.join(self.__resources_dir, DEFAULT_TEST_DIR)

  def cookie_setting_header(self):
    return dump_file(self.__test_files_dir, self.__test_file)

  def body_with_expectation(self):
    html_frame = dump_file(self.__resources_dir, EXPECTATION_HTML_SCAFFOLD)
    expected_data = dump_file(self.__test_files_dir, self.__expectation_file);
    return html_frame.format(**{'data' : expected_data})


def main(request, response):
  if "debug" in request.GET:
    response.writer.write_status(200)
    response.writer.end_headers()
    html_frame = dump_file(path.join(request.doc_root, DEFAULT_RESOURCE_DIR),
                           DEBUGGING_HTML_SCAFFOLD)
    test_file = html_frame % (request.GET['debug'])
    response.writer.write_content(test_file)
    return;

  if not "file" in request.GET:
    response.writer.write_status(404)
    response.writer.end_headers()
    response.writer.write_content("The 'file' parameter is missing!")
    return;

  cookie_response = CookieTestResponse(request.GET['file'], request.doc_root)

  response.writer.write_status(200)
  response.writer.write(cookie_response.cookie_setting_header())
  response.writer.end_headers()
  response.writer.write_content(cookie_response.body_with_expectation())
