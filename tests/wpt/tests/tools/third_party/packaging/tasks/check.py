# This file is dual licensed under the terms of the Apache License, Version
# 2.0, and the BSD License. See the LICENSE file in the root of this repository
# for complete details.

import itertools
import json
import os.path
import xmlrpc.client

import invoke
import pkg_resources
import progress.bar

from packaging.version import Version

from .paths import CACHE


def _parse_version(value):
    try:
        return Version(value)
    except ValueError:
        return None


@invoke.task
def pep440(cached=False):
    cache_path = os.path.join(CACHE, "pep440.json")

    # If we were given --cached, then we want to attempt to use cached data if
    # possible
    if cached:
        try:
            with open(cache_path) as fp:
                data = json.load(fp)
        except Exception:
            data = None
    else:
        data = None

    # If we don't have data, then let's go fetch it from PyPI
    if data is None:
        bar = progress.bar.ShadyBar("Fetching Versions")
        client = xmlrpc.client.Server("https://pypi.python.org/pypi")

        data = {
            project: client.package_releases(project, True)
            for project in bar.iter(client.list_packages())
        }

        os.makedirs(os.path.dirname(cache_path), exist_ok=True)
        with open(cache_path, "w") as fp:
            json.dump(data, fp)

    # Get a list of all of the version numbers on PyPI
    all_versions = list(itertools.chain.from_iterable(data.values()))

    # Determine the total number of versions which are compatible with the
    # current routine
    parsed_versions = [
        _parse_version(v) for v in all_versions if _parse_version(v) is not None
    ]

    # Determine a list of projects that sort exactly the same between
    # pkg_resources and PEP 440
    compatible_sorting = [
        project
        for project, versions in data.items()
        if (
            sorted(versions, key=pkg_resources.parse_version)
            == sorted((x for x in versions if _parse_version(x)), key=Version)
        )
    ]

    # Determine a list of projects that sort exactly the same between
    # pkg_resources and PEP 440 when invalid versions are filtered out
    filtered_compatible_sorting = [
        project
        for project, versions in (
            (p, [v for v in vs if _parse_version(v) is not None])
            for p, vs in data.items()
        )
        if (
            sorted(versions, key=pkg_resources.parse_version)
            == sorted(versions, key=Version)
        )
    ]

    # Determine a list of projects which do not have any versions that are
    # valid with PEP 440 and which have any versions registered
    only_invalid_versions = [
        project
        for project, versions in data.items()
        if (versions and not [v for v in versions if _parse_version(v) is not None])
    ]

    # Determine a list of projects which have matching latest versions between
    # pkg_resources and PEP 440
    differing_latest_versions = [
        project
        for project, versions in data.items()
        if (
            sorted(versions, key=pkg_resources.parse_version)[-1:]
            != sorted((x for x in versions if _parse_version(x)), key=Version)[-1:]
        )
    ]

    # Print out our findings
    print(
        "Total Version Compatibility:              {}/{} ({:.2%})".format(
            len(parsed_versions),
            len(all_versions),
            len(parsed_versions) / len(all_versions),
        )
    )
    print(
        "Total Sorting Compatibility (Unfiltered): {}/{} ({:.2%})".format(
            len(compatible_sorting), len(data), len(compatible_sorting) / len(data)
        )
    )
    print(
        "Total Sorting Compatibility (Filtered):   {}/{} ({:.2%})".format(
            len(filtered_compatible_sorting),
            len(data),
            len(filtered_compatible_sorting) / len(data),
        )
    )
    print(
        "Projects with No Compatible Versions:     {}/{} ({:.2%})".format(
            len(only_invalid_versions),
            len(data),
            len(only_invalid_versions) / len(data),
        )
    )
    print(
        "Projects with Differing Latest Version:   {}/{} ({:.2%})".format(
            len(differing_latest_versions),
            len(data),
            len(differing_latest_versions) / len(data),
        )
    )
