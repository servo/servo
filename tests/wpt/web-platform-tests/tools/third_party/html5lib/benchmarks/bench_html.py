import io
import os
import sys

import pyperf

sys.path[0:0] = [os.path.join(os.path.dirname(__file__), "..")]
import html5lib  # noqa: E402


def bench_parse(fh, treebuilder):
    fh.seek(0)
    html5lib.parse(fh, treebuilder=treebuilder, useChardet=False)


def bench_serialize(loops, fh, treebuilder):
    fh.seek(0)
    doc = html5lib.parse(fh, treebuilder=treebuilder, useChardet=False)

    range_it = range(loops)
    t0 = pyperf.perf_counter()

    for loops in range_it:
        html5lib.serialize(doc, tree=treebuilder, encoding="ascii", inject_meta_charset=False)

    return pyperf.perf_counter() - t0


BENCHMARKS = ["parse", "serialize"]


def add_cmdline_args(cmd, args):
    if args.benchmark:
        cmd.append(args.benchmark)


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.metadata["description"] = "Run benchmarks based on Anolis"
    runner.argparser.add_argument("benchmark", nargs="?", choices=BENCHMARKS)

    args = runner.parse_args()
    if args.benchmark:
        benchmarks = (args.benchmark,)
    else:
        benchmarks = BENCHMARKS

    with open(os.path.join(os.path.dirname(__file__), "data", "html.html"), "rb") as fh:
        source = io.BytesIO(fh.read())

    if "parse" in benchmarks:
        for tb in ("etree", "dom", "lxml"):
            runner.bench_func("html_parse_%s" % tb, bench_parse, source, tb)

    if "serialize" in benchmarks:
        for tb in ("etree", "dom", "lxml"):
            runner.bench_time_func("html_serialize_%s" % tb, bench_serialize, source, tb)
