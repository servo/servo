# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

def create_parser():
    from wptrunner import wptcommandline

    parser = wptcommandline.create_parser_update()
    parser.add_argument("--upstream", dest="upstream", action="store_true", default=None,
                        help="Push local changes to upstream repository even when not syncing")
    parser.add_argument("--no-upstream", dest="upstream", action="store_false", default=None,
                        help="Dont't push local changes to upstream repository when syncing")
    parser.add_argument("--token-file", action="store", type=wptcommandline.abs_path,
                        help="Path to file containing github token")
    parser.add_argument("--token", action="store", help="GitHub token to use")
    return parser


def check_args(kwargs):
    from wptrunner import wptcommandline

    wptcommandline.set_from_config(kwargs)
    kwargs["upstream"] = kwargs["upstream"] if kwargs["upstream"] is not None else kwargs["sync"]

    if kwargs["upstream"]:
        if kwargs["rev"]:
            raise ValueError("Setting --rev with --upstream isn't supported")
        if kwargs["token"] is None:
            if kwargs["token_file"] is None:
                raise ValueError("Must supply either a token file or a token")
            with open(kwargs["token_file"]) as f:
                token = f.read().strip()
                kwargs["token"] = token
    del kwargs["token_file"]
    return kwargs

def parse_args():
    parser = create_parser()
    kwargs = vars(parser.parse_args())
    return check_args(kwargs)
