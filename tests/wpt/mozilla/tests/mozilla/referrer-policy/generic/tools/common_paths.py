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

selection_pattern = '%(delivery_method)s/' + \
                    '%(origin)s/' + \
                    '%(source_protocol)s-%(target_protocol)s/' + \
                    '%(subresource)s/'

test_file_path_pattern = '%(spec_name)s/' + selection_pattern + \
                         '%(name)s.%(redirection)s.%(source_protocol)s.html'


def get_template(basename):
    with open(os.path.join(template_directory, basename)) as f:
        return f.read()


def read_nth_line(fp, line_number):
  fp.seek(0)
  for i, line in enumerate(fp):
    if (i + 1) == line_number:
      return line


def load_spec_json():
    re_error_location = re.compile('line ([0-9]+) column ([0-9]+)')
    with open(spec_filename) as f:
        try:
          spec_json = json.load(f)
        except ValueError, ex:
          print ex.message
          match = re_error_location.search(ex.message)
          if match:
            line_number, column = int(match.group(1)), int(match.group(2))
            print read_nth_line(f, line_number).rstrip()
            print " " * (column - 1) + "^"

          sys.exit(1)

        return spec_json
