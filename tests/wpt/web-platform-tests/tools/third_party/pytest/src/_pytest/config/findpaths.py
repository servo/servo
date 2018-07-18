import py
import os
from .exceptions import UsageError


def exists(path, ignore=EnvironmentError):
    try:
        return path.check()
    except ignore:
        return False


def getcfg(args, warnfunc=None):
    """
    Search the list of arguments for a valid ini-file for pytest,
    and return a tuple of (rootdir, inifile, cfg-dict).

    note: warnfunc is an optional function used to warn
        about ini-files that use deprecated features.
        This parameter should be removed when pytest
        adopts standard deprecation warnings (#1804).
    """
    from _pytest.deprecated import CFG_PYTEST_SECTION

    inibasenames = ["pytest.ini", "tox.ini", "setup.cfg"]
    args = [x for x in args if not str(x).startswith("-")]
    if not args:
        args = [py.path.local()]
    for arg in args:
        arg = py.path.local(arg)
        for base in arg.parts(reverse=True):
            for inibasename in inibasenames:
                p = base.join(inibasename)
                if exists(p):
                    iniconfig = py.iniconfig.IniConfig(p)
                    if "pytest" in iniconfig.sections:
                        if inibasename == "setup.cfg" and warnfunc:
                            warnfunc(
                                "C1", CFG_PYTEST_SECTION.format(filename=inibasename)
                            )
                        return base, p, iniconfig["pytest"]
                    if (
                        inibasename == "setup.cfg"
                        and "tool:pytest" in iniconfig.sections
                    ):
                        return base, p, iniconfig["tool:pytest"]
                    elif inibasename == "pytest.ini":
                        # allowed to be empty
                        return base, p, {}
    return None, None, None


def get_common_ancestor(paths):
    common_ancestor = None
    for path in paths:
        if not path.exists():
            continue
        if common_ancestor is None:
            common_ancestor = path
        else:
            if path.relto(common_ancestor) or path == common_ancestor:
                continue
            elif common_ancestor.relto(path):
                common_ancestor = path
            else:
                shared = path.common(common_ancestor)
                if shared is not None:
                    common_ancestor = shared
    if common_ancestor is None:
        common_ancestor = py.path.local()
    elif common_ancestor.isfile():
        common_ancestor = common_ancestor.dirpath()
    return common_ancestor


def get_dirs_from_args(args):

    def is_option(x):
        return str(x).startswith("-")

    def get_file_part_from_node_id(x):
        return str(x).split("::")[0]

    def get_dir_from_path(path):
        if path.isdir():
            return path
        return py.path.local(path.dirname)

    # These look like paths but may not exist
    possible_paths = (
        py.path.local(get_file_part_from_node_id(arg))
        for arg in args
        if not is_option(arg)
    )

    return [get_dir_from_path(path) for path in possible_paths if path.exists()]


def determine_setup(inifile, args, warnfunc=None, rootdir_cmd_arg=None):
    dirs = get_dirs_from_args(args)
    if inifile:
        iniconfig = py.iniconfig.IniConfig(inifile)
        is_cfg_file = str(inifile).endswith(".cfg")
        # TODO: [pytest] section in *.cfg files is depricated. Need refactoring.
        sections = ["tool:pytest", "pytest"] if is_cfg_file else ["pytest"]
        for section in sections:
            try:
                inicfg = iniconfig[section]
                if is_cfg_file and section == "pytest" and warnfunc:
                    from _pytest.deprecated import CFG_PYTEST_SECTION

                    warnfunc("C1", CFG_PYTEST_SECTION.format(filename=str(inifile)))
                break
            except KeyError:
                inicfg = None
        rootdir = get_common_ancestor(dirs)
    else:
        ancestor = get_common_ancestor(dirs)
        rootdir, inifile, inicfg = getcfg([ancestor], warnfunc=warnfunc)
        if rootdir is None:
            for rootdir in ancestor.parts(reverse=True):
                if rootdir.join("setup.py").exists():
                    break
            else:
                rootdir, inifile, inicfg = getcfg(dirs, warnfunc=warnfunc)
                if rootdir is None:
                    rootdir = get_common_ancestor([py.path.local(), ancestor])
                    is_fs_root = os.path.splitdrive(str(rootdir))[1] == "/"
                    if is_fs_root:
                        rootdir = ancestor
    if rootdir_cmd_arg:
        rootdir_abs_path = py.path.local(os.path.expandvars(rootdir_cmd_arg))
        if not os.path.isdir(str(rootdir_abs_path)):
            raise UsageError(
                "Directory '{}' not found. Check your '--rootdir' option.".format(
                    rootdir_abs_path
                )
            )
        rootdir = rootdir_abs_path
    return rootdir, inifile, inicfg or {}
