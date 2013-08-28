/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Timing functions.
use extra::time::precise_time_ns;
use std::cell::Cell;
use std::comm::{Port, SharedChan};
use extra::sort::tim_sort;
use std::iterator::AdditiveIterator;

// front-end representation of the profiler used to communicate with the profiler
#[deriving(Clone)]
pub struct ProfilerChan {
    chan: SharedChan<ProfilerMsg>,
}

impl ProfilerChan {
    pub fn new(chan: Chan<ProfilerMsg>) -> ProfilerChan {
        ProfilerChan {
            chan: SharedChan::new(chan),
        }
    }
    pub fn send(&self, msg: ProfilerMsg) {
        self.chan.send(msg);
    }
}

#[deriving(Eq, Clone)]
pub enum ProfilerCategory {
    CompositingCategory,
    LayoutQueryCategory,
    LayoutPerformCategory,
    LayoutAuxInitCategory,
    LayoutSelectorMatchCategory,
    LayoutTreeBuilderCategory,
    LayoutMainCategory,
    LayoutShapingCategory,
    LayoutDispListBuildCategory,
    GfxRegenAvailableFontsCategory,
    RenderingDrawingCategory,
    RenderingPrepBuffCategory,
    RenderingCategory,
    // FIXME(rust#8803): workaround for lack of CTFE function on enum types to return length
    NumBuckets,
}
struct ProfilerBucket {
    category: ProfilerCategory,
    data: ~[float],
}
impl ProfilerBucket {
    fn new(category: ProfilerCategory) -> ProfilerBucket {
        ProfilerBucket {
            category: category,
            data: ~[],
        }
    }
}
// FIXME(rust#5873) this should be initialized by a NumBuckets cast
type ProfilerBuckets = [ProfilerBucket, ..13];

pub enum ProfilerMsg {
    // Normal message used for reporting time
    TimeMsg(ProfilerCategory, float),
    // Message used to force print the profiling metrics
    PrintMsg,
}

// back end of the profiler that handles data aggregation and performance metrics
pub struct Profiler {
    port: Port<ProfilerMsg>,
    buckets: ProfilerBuckets,
    last_msg: Option<ProfilerMsg>,
}

impl ProfilerCategory {
    // convenience function to not have to cast every time
    pub fn num_buckets() -> uint {
        NumBuckets as uint
    }

    // enumeration of all ProfilerCategory types
    fn empty_buckets() -> ProfilerBuckets {
        [
            ProfilerBucket::new(CompositingCategory),
            ProfilerBucket::new(LayoutQueryCategory),
            ProfilerBucket::new(LayoutPerformCategory),
            ProfilerBucket::new(LayoutAuxInitCategory),
            ProfilerBucket::new(LayoutSelectorMatchCategory),
            ProfilerBucket::new(LayoutTreeBuilderCategory),
            ProfilerBucket::new(LayoutMainCategory),
            ProfilerBucket::new(LayoutShapingCategory),
            ProfilerBucket::new(LayoutDispListBuildCategory),
            ProfilerBucket::new(GfxRegenAvailableFontsCategory),
            ProfilerBucket::new(RenderingDrawingCategory),
            ProfilerBucket::new(RenderingPrepBuffCategory),
            ProfilerBucket::new(RenderingCategory),
        ]
    }

    // some categories are subcategories of LayoutPerformCategory
    // and should be printed to indicate this
    pub fn format(self) -> ~str {
        let padding = match self {
            LayoutAuxInitCategory | LayoutSelectorMatchCategory | LayoutTreeBuilderCategory |
            LayoutMainCategory | LayoutDispListBuildCategory | LayoutShapingCategory=> " - ",
            _ => ""
        };
        fmt!("%s%?", padding, self)
    }
}

impl Profiler {
    pub fn create(port: Port<ProfilerMsg>) {
        let port = Cell::new(port);
        do spawn {
            let mut profiler = Profiler::new(port.take());
            profiler.start();
        }
    }

    pub fn new(port: Port<ProfilerMsg>) -> Profiler {
        Profiler {
            port: port,
            buckets: ProfilerCategory::empty_buckets(),
            last_msg: None,
        }
    }

    pub fn start(&mut self) {
        loop {
            let msg = self.port.try_recv();
            match msg {
               Some (msg) => self.handle_msg(msg),
               None => break
            }
        }
    }

    fn handle_msg(&mut self, msg: ProfilerMsg) {
        match msg {
            TimeMsg(category, t) => self.buckets[category as uint].data.push(t),
            PrintMsg => match self.last_msg {
                // only print if more data has arrived since the last printout
                Some(TimeMsg(*)) => self.print_buckets(),
                _ => ()
            },
        };
        self.last_msg = Some(msg);
    }

    fn print_buckets(&mut self) {
        println(fmt!("%31s %15s %15s %15s %15s %15s",
                         "_category_", "_mean (ms)_", "_median (ms)_",
                         "_min (ms)_", "_max (ms)_", "_bucket size_"));
        for &ProfilerBucket { category: ref category, data: ref mut data } in self.buckets.mut_iter() {
            tim_sort(*data);
            let data_len = data.len();
            if data_len > 0 {
                let (mean, median, &min, &max) =
                    (data.iter().map(|&x|x).sum() / (data_len as float),
                     data[data_len / 2],
                     data.iter().min().unwrap(),
                     data.iter().max().unwrap());
                println(fmt!("%-30s: %15.4f %15.4f %15.4f %15.4f %15u",
                             category.format(), mean, median, min, max, data_len));
            }
        }
        println("");
    }
}


pub fn profile<T>(category: ProfilerCategory, 
                  profiler_chan: ProfilerChan,
                  callback: &fn() -> T)
                  -> T {
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) as float / 1000000f);
    profiler_chan.send(TimeMsg(category, ms));
    return val;
}

pub fn time<T>(msg: &str, callback: &fn() -> T) -> T{
    let start_time = precise_time_ns();
    let val = callback();
    let end_time = precise_time_ns();
    let ms = ((end_time - start_time) as float / 1000000f);
    if ms >= 5f {
        debug!("%s took %? ms", msg, ms);
    }
    return val;
}

#[cfg(test)]
mod test {
    // ensure that the order of the buckets matches the order of the enum categories
    #[test]
    fn check_order() {
        let buckets = ProfilerCategory::empty_buckets();
        for (i, bucket) in buckets.iter().enumerate() {
            assert!(bucket.category as uint == i);
        }
    }
}
