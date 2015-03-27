import argparse
import sys

from .. import handlers, commandline, reader

def get_parser(add_help=True):
    parser = argparse.ArgumentParser("format",
                                     description="Format a structured log stream", add_help=add_help)
    parser.add_argument("--input", action="store", default=None,
                        help="Filename to read from, defaults to stdin")
    parser.add_argument("--output", action="store", default=None,
                        help="Filename to write to, defaults to stdout")
    parser.add_argument("format", choices=commandline.log_formatters.keys(),
                        help="Format to use")
    return parser

def main(**kwargs):
    if kwargs["input"] is None:
        input_file = sys.stdin
    else:
        input_file = open(kwargs["input"])
    if kwargs["output"] is None:
        output_file = sys.stdout
    else:
        output_file = open(kwargs["output"], "w")

    formatter = commandline.log_formatters[kwargs["format"]][0]()

    handler = handlers.StreamHandler(stream=output_file,
                                     formatter=formatter)

    for data in reader.read(input_file):
        handler(data)

if __name__ == "__main__":
    parser = get_parser()
    args = parser.parse_args()
    kwargs = vars(args)
    main(**kwargs)
