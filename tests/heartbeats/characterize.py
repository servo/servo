#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import sys
import os
from os import path
import time
import datetime
import argparse
import platform
import subprocess

TOP_DIR = path.join("..", "..")
GUARD_TIME = 10
HEARTBEAT_DEFAULT_WINDOW_SIZE = 20
# Use a larger window sizes to reduce or prevent writing log files until benchmark completion
# (profiler name, window size)
# These categories need to be kept aligned with ProfilerCategory in components/profile_traits/time.rs
HEARTBEAT_PROFILER_CATEGORIES = [
    ("Compositing", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutPerform", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutStyleRecalc", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    # ("LayoutTextShaping", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutRestyleDamagePropagation", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutNonIncrementalReset", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutSelectorMatch", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutTreeBuilder", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutDamagePropagate", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutGeneratedContent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutDisplayListSorting", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutFloatPlacementSpeculation", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutMain", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutStoreOverflow", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutParallelWarmup", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("LayoutDispListBuild", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("NetHTTPRequestResponse", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("PaintingPerTile", 50),
    ("PaintingPrepBuff", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("Painting", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ImageDecoding", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ImageSaving", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptAttachLayout", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptConstellationMsg", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptDevtoolsMsg", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptDocumentEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptDomEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptEvaluate", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptFileRead", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptImageCacheMsg", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptInputEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptNetworkEvent", 200),
    ("ScriptParseHTML", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptPlannedNavigation", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptResize", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptSetScrollState", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptSetViewport", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptTimerEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptStylesheetLoad", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptUpdateReplacedElement", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptWebSocketEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptWorkerEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptServiceWorkerEvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptParseXML", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptEnterFullscreen", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptExitFullscreen", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ScriptWebVREvent", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("ApplicationHeartbeat", 100),
    ("WebGlGetContextAttributes", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlActiveTexture", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBlendColor", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBlendEquation", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBlendEquationSeparate", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBlendFunc", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBlendFuncSeparate", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlAttachShader", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDetachShader", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBindAttribLocation", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBufferData", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBufferSubData", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlClear", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlClearColor", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlClearDepth", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlClearStencil", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlColorMask", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCullFace", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlFrontFace", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDepthFunc", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDepthMask", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDepthRange", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlEnable", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDisable", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCompileShader", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCopyTexImage2D", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCopyTexSubImage2D", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCreateBuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCreateFramebuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCreateRenderbuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCreateTexture", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCreateProgram", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCreateShader", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDeleteBuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDeleteFramebuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDeleteRenderbuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDeleteTexture", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDeleteProgram", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDeleteShader", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBindBuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBindFramebuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBindRenderbuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBindTexture", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDisableVertexAttribArray", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlEnableVertexAttribArray", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlFramebufferRenderbuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlFramebufferTexture2D", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetExtensions", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetShaderPrecisionFormat", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformLocation", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetShaderInfoLog", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetProgramInfoLog", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetFramebufferAttachmentParameter", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetRenderbufferParameter", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlPolygonOffset", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlRenderbufferStorage", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlReadPixels", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlSampleCoverage", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlScissor", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlStencilFunc", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlStencilFuncSeparate", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlStencilMask", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlStencilMaskSeparate", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlStencilOp", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlStencilOpSeparate", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlHint", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlLineWidth", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlPixelStorei", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlLinkProgram", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform1f", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform1fv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform1i", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform1iv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform2f", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform2fv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform2i", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform2iv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform3f", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform3fv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform3i", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform3iv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform4f", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform4fv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform4i", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniform4iv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniformMatrix2fv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniformMatrix3fv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUniformMatrix4fv", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlUseProgram", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlValidateProgram", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlVertexAttrib", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlVertexAttribPointer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlVertexAttribPointer2f", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlSetViewport", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlTexImage2D", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlTexSubImage2D", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDrawingBufferWidth", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDrawingBufferHeight", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlFinish", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlFlush", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGenerateMipmap", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlCreateVertexArray", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDeleteVertexArray", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlBindVertexArray", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterBool", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterBool4", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterInt", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterInt2", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterInt4", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterFloat", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterFloat2", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetParameterFloat4", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetProgramValidateStatus", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetProgramActiveUniforms", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetCurrentVertexAttrib", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetTexParameterFloat", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetTexParameterInt", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlTexParameteri", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlTexParameterf", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDrawArrays", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDrawArraysInstanced", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDrawElements", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDrawElementsInstanced", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlVertexAttribDivisor", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformBool", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformBool2", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformBool3", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformBool4", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformInt", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformInt2", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformInt3", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformInt4", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformFloat", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformFloat2", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformFloat3", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformFloat4", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformFloat9", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlGetUniformFloat16", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlInitializeFramebuffer", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMFinish", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMDrawingBufferWidth", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMDrawingBufferHeight", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetParameter", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetTexParameter", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetContextAttributes", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetFramebufferAttachmentParameter", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetRenderbufferParameter", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetProgramParameter", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetShaderPrecisionFormat", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetVertexAttrib", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMReadPixels", HEARTBEAT_DEFAULT_WINDOW_SIZE),
    ("WebGlDOMGetUniform", HEARTBEAT_DEFAULT_WINDOW_SIZE),
]
ENERGY_READER_BIN = "energymon-file-provider"
ENERGY_READER_TEMP_OUTPUT = "energymon.txt"
SUMMARY_OUTPUT = "summary.txt"


def get_command(build_target, layout_thread_count, renderer, page, profile):
    """Get the command to execute.
    """
    return path.join(TOP_DIR, "target", build_target, "servo") + \
        " -p %d -o output.png -y %d %s -Z profile-script-events '%s'" % \
        (profile, layout_thread_count, renderer, page)


def set_app_environment(log_dir):
    """Set environment variables to enable heartbeats.
    """
    prefix = "heartbeat-"
    for (profiler, window) in HEARTBEAT_PROFILER_CATEGORIES:
        os.environ["SERVO_HEARTBEAT_ENABLE_" + profiler] = ""
        os.environ["SERVO_HEARTBEAT_LOG_" + profiler] = path.join(log_dir, prefix + profiler + ".log")
        os.environ["SERVO_HEARTBEAT_WINDOW_" + profiler] = str(window)


def start_energy_reader():
    """Energy reader writes to a file that we will poll.
    """
    os.system(ENERGY_READER_BIN + " " + ENERGY_READER_TEMP_OUTPUT + "&")


def stop_energy_reader():
    """Stop the energy reader and remove its temp file.
    """
    os.system("pkill -x " + ENERGY_READER_BIN)
    os.remove(ENERGY_READER_TEMP_OUTPUT)


def read_energy():
    """Poll the energy reader's temp file.
    """
    data = 0
    with open(ENERGY_READER_TEMP_OUTPUT, "r") as em:
        data = int(em.read().replace('\n', ''))
    return data


def git_rev_hash():
    """Get the git revision hash.
    """
    return subprocess.check_output(['git', 'rev-parse', 'HEAD']).rstrip()


def git_rev_hash_short():
    """Get the git revision short hash.
    """
    return subprocess.check_output(['git', 'rev-parse', '--short', 'HEAD']).rstrip()


def execute(base_dir, build_target, renderer, page, profile, trial, layout_thread_count):
    """Run a single execution.
    """
    log_dir = path.join(base_dir, "logs_l" + str(layout_thread_count),
                        "trial_" + str(trial))
    if os.path.exists(log_dir):
        print "Log directory already exists: " + log_dir
        sys.exit(1)
    os.makedirs(log_dir)

    set_app_environment(log_dir)
    cmd = get_command(build_target, layout_thread_count, renderer, page, profile)

    # Execute
    start_energy_reader()
    print 'sleep ' + str(GUARD_TIME)
    time.sleep(GUARD_TIME)
    time_start = time.time()
    energy_start = read_energy()
    print cmd
    os.system(cmd)
    energy_end = read_energy()
    time_end = time.time()
    stop_energy_reader()
    print 'sleep ' + str(GUARD_TIME)
    time.sleep(GUARD_TIME)

    uj = energy_end - energy_start
    latency = time_end - time_start
    watts = uj / 1000000.0 / latency
    # Write a file that describes this execution
    with open(path.join(log_dir, SUMMARY_OUTPUT), "w") as f:
        f.write("Datetime (UTC): " + datetime.datetime.utcnow().isoformat())
        f.write("\nPlatform: " + platform.platform())
        f.write("\nGit hash: " + git_rev_hash())
        f.write("\nGit short hash: " + git_rev_hash_short())
        f.write("\nRelease: " + build_target)
        f.write("\nLayout threads: " + str(layout_thread_count))
        f.write("\nTrial: " + str(trial))
        f.write("\nCommand: " + cmd)
        f.write("\nTime (sec): " + str(latency))
        f.write("\nEnergy (uJ): " + str(uj))
        f.write("\nPower (W): " + str(watts))


def characterize(build_target, base_dir, (min_layout_threads, max_layout_threads), renderer, page, profile, trials):
    """Run all configurations and capture results.
    """
    for layout_thread_count in xrange(min_layout_threads, max_layout_threads + 1):
        for trial in xrange(1, trials + 1):
            execute(base_dir, build_target, renderer, page, profile, trial, layout_thread_count)


def main():
    """For this script to be useful, the following conditions are needed:
    - HEARTBEAT_PROFILER_CATEGORIES should be aligned with the profiler categories in the source code.
    - The "energymon" project needs to be installed to the system (libraries and the "energymon" binary).
     - The "default" energymon library will be used - make sure you choose one that is useful for your system setup
       when installing energymon.
    - Build servo in release mode with the "energy-profiling" feature enabled (this links with the energymon lib).
    """
    # Default max number of layout threads
    max_layout_threads = 1
    # Default benchmark
    benchmark = path.join(TOP_DIR, "tests", "html", "perf-rainbow.html")
    # Default renderer
    renderer = ""
    # Default output directory
    output_dir = "heartbeat_logs"
    # Default build target
    build_target = "release"
    # Default profile interval
    profile = 60
    # Default single argument
    single = False
    # Default number of trials
    trials = 1

    # Parsing the input of the script
    parser = argparse.ArgumentParser(description="Characterize Servo timing and energy behavior")
    parser.add_argument("-b", "--benchmark",
                        default=benchmark,
                        help="Gets the benchmark, for example \"-b http://www.example.com\"")
    parser.add_argument("-d", "--debug",
                        action='store_true',
                        help="Use debug build instead of release build")
    parser.add_argument("-w", "--webrender",
                        action='store_true',
                        help="Use webrender backend")
    parser.add_argument("-l", "--max_layout_threads",
                        help="Specify the maximum number of threads for layout, for example \"-l 5\"")
    parser.add_argument("-o", "--output",
                        help="Specify the log output directory, for example \"-o heartbeat_logs\"")
    parser.add_argument("-p", "--profile",
                        default=60,
                        help="Profiler output interval, for example \"-p 60\"")
    parser.add_argument("-s", "--single",
                        action='store_true',
                        help="Just run a single trial of the config provided, for example \"-s\"")
    parser.add_argument("-t", "--trials",
                        default=1,
                        type=int,
                        help="Number of trials to run for each configuration, for example \"-t 1\"")

    args = parser.parse_args()
    if args.benchmark:
        benchmark = args.benchmark
    if args.debug:
        build_target = "debug"
    if args.webrender:
        renderer = "-w"
    if args.max_layout_threads:
        max_layout_threads = int(args.max_layout_threads)
    if args.output:
        output_dir = args.output
    if args.profile:
        profile = args.profile
    if args.single:
        single = True
    if args.trials:
        trials = args.trials

    if os.path.exists(output_dir):
        print "Output directory already exists: " + output_dir
        sys.exit(1)
    os.makedirs(output_dir)

    if single:
        execute(output_dir, build_target, renderer, benchmark, profile, trials, max_layout_threads)
    else:
        characterize(build_target, output_dir, (1, max_layout_threads), renderer, benchmark, profile, trials)

if __name__ == "__main__":
    main()
