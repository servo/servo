/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use criterion::*;
use pixels::Multiply;

const PIXELS: usize = 1_000_000;

fn create_data(alpha: impl Fn(usize) -> u8) -> Vec<u8> {
    (0..PIXELS)
        .flat_map(|i| {
            let color = i as u8;
            [color, !color, color / 3, alpha(i)]
        })
        .collect()
}

fn bench(c: &mut Criterion) {
    let inputs = [
        ("opaque", create_data(|_| u8::MAX)),
        ("transparent", create_data(|_| 0)),
        // Every alpha that needs a division, in turn.
        (
            "translucent",
            create_data(|i| (i % usize::from(u8::MAX - 1)) as u8 + 1),
        ),
    ];

    let mut group = c.benchmark_group("unmultiply_inplace");
    for (name, data) in &inputs {
        group.bench_function(*name, |b| {
            b.iter_batched_ref(
                || data.clone(),
                |data| pixels::unmultiply_inplace::<false>(data),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();

    let mut group = c.benchmark_group("transform_inplace_unmultiply");
    for (name, data) in &inputs {
        group.bench_function(*name, |b| {
            b.iter_batched_ref(
                || data.clone(),
                |data| pixels::transform_inplace(data, Multiply::UnMultiply, false, false),
                BatchSize::LargeInput,
            )
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
