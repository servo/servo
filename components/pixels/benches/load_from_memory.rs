/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::Path;

use criterion::*;
use pixels::{CorsStatus, load_from_memory};

/// Blink's image decoder perf test images.
const RESOURCES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/blink_perf_tests/perf_tests/image_decoder/resources"
);

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_from_memory");
    group.sampling_mode(SamplingMode::Flat).sample_size(20);
    // `palette.png` has transparency, so it is still premultiplied.
    for name in [
        "droids-h2v2.jpg",
        "droids.png",
        "droids.webp",
        "palette.png",
    ] {
        let data = std::fs::read(Path::new(RESOURCES).join(name)).expect("image is in the tree");
        group.bench_function(name, |b| {
            b.iter(|| load_from_memory(&data, CorsStatus::Safe))
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
