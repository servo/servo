import os, sys, json, re

script_directory = os.path.dirname(os.path.abspath(__file__))
generic_directory = os.path.abspath(os.path.join(script_directory, '..'))

template_directory = os.path.abspath(os.path.join(script_directory,
                                                  '..',
                                                  'template'))
spec_directory = os.path.abspath(os.path.join(script_directory, '..', '..'))
test_root_directory = os.path.abspath(os.path.join(script_directory,
                                                   '..', '..', '..'))

spec_filename = os.path.join(spec_directory, "spec.src.json")
generated_spec_json_filename = os.path.join(spec_directory, "spec_json.js")

selection_pattern = '%(subresource)s/' + \
                    '%(opt_in_method)s/' + \
                    '%(origin)s/' + \
                    '%(context_nesting)s/' + \
                    '%(redirection)s/'

test_file_path_pattern = selection_pattern + \
                         '%(spec_name)s/' + \
                         '%(name)s.%(source_scheme)s.html'


def get_template(basename):
    with open(os.path.join(template_directory, basename), "r") as f:
        return f.read()


def write_file(filename, contents):
    with open(filename, "w") as f:
        f.write(contents)


def read_nth_line(fp, line_number):
  fp.seek(0)
  for i, line in enumerate(fp):
    if (i + 1) == line_number:
      return line


def load_spec_json(path_to_spec = None):
    if path_to_spec is None:
      path_to_spec = spec_filename

    re_error_location = re.compile('line ([0-9]+) column ([0-9]+)')
    with open(path_to_spec, "r") as f:
        try:
          return json.load(f)
        except ValueError, ex:
          print ex.message
          match = re_error_location.search(ex.message)
          if match:
            line_number, column = int(match.group(1)), int(match.group(2))
            print read_nth_line(f, line_number).rstrip()
            print " " * (column - 1) + "^"
          sys.exit(1)
