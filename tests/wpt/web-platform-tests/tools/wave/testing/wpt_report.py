# mypy: allow-untyped-defs

import subprocess
import os
import ntpath
import sys
from shutil import copyfile


def generate_report(
        input_json_directory_path=None,
        output_html_directory_path=None,
        spec_name=None,
        is_multi=None,
        reference_dir=None):
    if is_multi is None:
        is_multi = False
    try:
        command = [
            "wptreport",
            "--input", input_json_directory_path,
            "--output", output_html_directory_path,
            "--spec", spec_name,
            "--sort", "true",
            "--failures", "true",
            "--tokenFileName", "true" if is_multi else "false",
            "--pass", "100",
            "--ref", reference_dir if reference_dir is not None else ""]
        whole_command = ""
        for command_part in command:
            whole_command += command_part + " "
        subprocess.call(command, shell=False)
    except subprocess.CalledProcessError as e:
        info = sys.exc_info()
        raise Exception("Failed to execute wptreport: " + str(info[0].__name__) + ": " + e.output)


def generate_multi_report(
        output_html_directory_path=None,
        spec_name=None,
        result_json_files=None,
        reference_dir=None):
    for file in result_json_files:
        if not os.path.isfile(file["path"]):
            continue
        file_name = ntpath.basename(file["path"])
        copyfile(file["path"], os.path.join(
            output_html_directory_path,
            file["token"] + "-" + file_name
        ))

    generate_report(
        input_json_directory_path=output_html_directory_path,
        output_html_directory_path=output_html_directory_path,
        spec_name=spec_name,
        is_multi=True,
        reference_dir=reference_dir)
