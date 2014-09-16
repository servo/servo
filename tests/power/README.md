The power and performance measurement for Servo parallel browser

This script uses PowerMetrics to measure power usage of Servo on OS X

## Running

``` sh
cd servo/tests/power
sudo python PowerMeasure.py
```
You can define the maximum number of threads in layout level, rendering by cpu, benchmarks and output directory with these command line arguments:

- `-b BENCHMARK, --benchmark BENCHMARK` sets the benchmark, for example '-B "perf-rainbow.html"'
- `-c CPU, --CPU CPU` renders with CPU instead of GPU
- `-l LAYOUTTHREADS, --LayoutThreads LAYOUTTHREADS` sets the maximum number of threads for layout, for example " -L 5"
- `-o OUTPUT, --Output OUTPUT` specifyes the output directory

## Example

This command will measure power and performance for 1 to 5 threads in layout with CPU rendering when we are running the about-mozilla.html benchmark

``` sh
sudo python PowerMeasure.py -L 5 -c cpu -b "/Desktop/servo/src/test/html/about-mozilla.html" -o /Desktop/Results/
```
