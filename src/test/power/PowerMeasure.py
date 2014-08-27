# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# !/usr/bin/env python

# ---------Power measurement ------------------------------#
# This script will run the servo with the given benchmark and
# get the power usage using Powermetrics. Results will be put
# in sperate files with that name.
# Do not forget to run the script in servo/src/test/power folder
# --------------------------------------------------------#

import os
import time
import argparse

# ------------------------PowerCollector----------------------------#
# Collecting all the power data and put them into files


def PowerCollector(OutputDir, Benchmarks, LayoutThreads, Renderer):
    print " Running  the power collector"
    os.mkdir(os.path.join(OutputDir+"power"), 0777)
    os.mkdir(os.path.join(OutputDir+"time"), 0777)
    os.mkdir(os.path.join(OutputDir+"Addtional"), 0777)
    SleepTime = 20
    GuardTime = 0.5
    powerTiming = 1
    ExperimentNum = 21
    for ExpNum in range(1, ExperimentNum):
        for layoutT in range(1, LayoutThreads+1):
            PowerFiles = OutputDir + 'power/' + 'power-Layout' + \
                str(layoutT) + "-set" + str(ExpNum) + ".csv"
            TimeFiles = OutputDir + 'time/' + "time-Layout" + \
                str(layoutT) + "-set" + str(ExpNum) + ".csv"
            # ServoCmd ="(time ./servo -x -y " + str(layoutT) \
            #    +" "+ Renderer + " " + Benchmarks + " ) 2> " + TimeFiles
            ServoCmd = "(time ../../../build/servo -x -y " + \
                str(layoutT) + " " + Renderer + " " + \
                Benchmarks + " ) 2> " + TimeFiles
            Metrics = OutputDir + 'Addtional/' + "metrics-Layout" + \
                str(layoutT) + "-set" + str(ExpNum) + "-css.csv"
            cmd = '(sudo powermetrics -i ' + str(powerTiming) + \
                ' | grep \"energy\\|elapsed\\|servo\" > ' + \
                PowerFiles + '& ) 2> ' + Metrics
            time.sleep(SleepTime)
            os.system(cmd)
            time.sleep(GuardTime)
            os.system(ServoCmd)
            time.sleep(GuardTime)
            os.system('sudo pkill -9 powermetrics')
            time.sleep(SleepTime)

# -------------------PowerParser ---------------------------------#
# Parsing collected power by PowerCollector fucntion


def PowerParser(OutputDir, LayoutThreads):
    print "Running the PowerParser"
    ExperimentNum = 21
    ResultTable = OutputDir + "ResultTable.csv"
    ResultFile = open(ResultTable, "w")
    ResultFile.write("LayoutThreads, MeanPower, MeanTime , MaxTime, "
                     "MinTime , MaxPower , MinPower \n")

    for layoutT in range(1, LayoutThreads+1):
        MaxTime = 0
        MinTime = 1000000
        MaxPower = 0
        MinPower = 1000000
        TotalPower = 0
        TotalTime = 0
        for ExpNum in range(1, ExperimentNum):
            Files = OutputDir + 'power/' + 'power-Layout' + str(layoutT) + \
                "-set" + str(ExpNum) + ".csv"
            NewFile = OutputDir + 'power/Servo-L' + str(layoutT) + \
                "set" + str(ExpNum) + ".csv"
            File = open(Files, 'r')
            PowerFile = open(NewFile, 'w')
            TimeFiles = OutputDir + 'time/' + "time-Layout" + \
                str(layoutT) + "-set" + str(ExpNum) + ".csv"
            # ----Putting the power the power and its time into a table---- #

            for line in File:
                words = line.split()
                if words[0] == "***":
                    insertingWord = words[10][1:-2] + " "
                elif words[0] == "Intel":
                    insertingWord += words[7][:-1]
                    insertingWord += "\n"
                    PowerFile.write(insertingWord)
            File.close()
            PowerFile.close()

            # ---------------geting the total power of experiments-------- #

            TempFile = open(NewFile, 'r')
            Power = 0
            for line in TempFile:
                words2 = line.split()
                Power += float(words2[0]) * float(words2[1])
            TotalPower = float(Power / 1000.0)
            if TotalPower > MaxPower:
                MaxPower = TotalPower
            if TotalPower < MinPower:
                MinPower = TotalPower

            # -------------getting the total time of execution---------- #

            TempFile2 = open(TimeFiles, "r")
            for line in TempFile2:
                words3 = line.split()
                if line != "\n" and words3[0] == "real":
                    TotalTime = (float(words3[1][0]) * 60) + \
                        float(words3[1][2:-1])
            if TotalTime > MaxTime:
                MaxTime = TotalTime
            if TotalTime < MinTime:
                MinTime = TotalTime

        TotalPower = TotalPower / float(ExperimentNum-1)
        TotalTime = TotalTime / float(ExperimentNum-1)
        ResultFile.write(str(layoutT) + " , " + str(TotalPower) + " , " +
                         str(TotalTime) + " , " + str(MaxTime) + " , " +
                         str(MinTime) + " , " + str(MaxPower) + " , " +
                         str(MinPower) + "\n")
    ResultFile.close()
    Opener = ResultFile = open(ResultTable, "r")
    for line in Opener:
        print line

    print "Also you can find all the numbers for Power " \
        "and Performance in : ", ResultTable


# ----------------------------------------------------#
def main():
    LayoutThreads = 8  # Maximum number of threads considered for Layout
    Benchmarks = "../../test/html/perf-rainbow.html"
    OutputDir = "Experiments/"
    os.mkdir(os.path.join(OutputDir), 0777)
    Renderer = " "

    # Parsing the input of the script
    parser = argparse.ArgumentParser(description="Mesuring \
        power and performance of your servo runs")
    parser.add_argument("-b", "--benchmark", help="Gets the \
        benchmark, for example \"-B perf-rainbow.html\"")
    parser.add_argument("-c", "--CPU", help="Rendering with \
        CPU instead of  GPU, for example -C")
    parser.add_argument("-l", "--LayoutThreads", help="Specifyes \
        the maximum number of threads for layout, for example \" -L 5\"")
    parser.add_argument("-o", "--Output", help=" Specifyes \
        the output directory")

    args = parser.parse_args()
    if args.benchmark:
        Benchmarks = args.benchmark
    if args.CPU:
        Renderer = "-c"
    if args.LayoutThreads:
        LayoutThreads = int(args.LayoutThreads)
    if args.Output:
        OutputDir = args.Output

    PowerCollector(OutputDir, Benchmarks, LayoutThreads, Renderer)
    PowerParser(OutputDir, LayoutThreads)

if __name__ == "__main__":
    main()
