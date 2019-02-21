/*
 * Copyright 2019 Zejun Li
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::iter;
use std::path::Path;

use criterion::{criterion_group, criterion_main, Criterion};
use serde::*;

use kaosu_packer::geom::Cuboid;
use kaosu_packer::*;

criterion_group!(benches, pack_easy, pack_medium, pack_hard);
criterion_main!(benches);

fn pack_easy(c: &mut Criterion) {
    let items = load_items("testdata/easy.csv");
    let params = Params::default();
    let bin = Cuboid::new(30, 30, 30);
    c.bench_function("pack_easy", move |b| {
        b.iter(|| {
            pack_boxes(params, bin, &items);
        })
    });
}

fn pack_medium(c: &mut Criterion) {
    let items = load_items("testdata/medium.csv");
    let params = Params::default();
    let bin = Cuboid::new(100, 100, 100);
    c.bench_function("pack_medium", move |b| {
        b.iter(|| {
            pack_boxes(params, bin, &items);
        })
    });
}

fn pack_hard(c: &mut Criterion) {
    let items = load_items("testdata/hard.csv");
    let params = Params::default();
    let bin = Cuboid::new(100, 100, 100);
    c.bench_function("pack_hard", move |b| {
        b.iter(|| {
            pack_boxes(params, bin, &items);
        })
    });
}

#[derive(Debug, Deserialize)]
struct Record {
    width: i32,
    depth: i32,
    height: i32,
    count: usize,
}

fn load_items<P: AsRef<Path>>(path: P) -> Vec<Cuboid> {
    let mut rdr = csv::Reader::from_path(path).unwrap();
    let mut v = Vec::new();
    for record in rdr.deserialize() {
        let record: Record = record.unwrap();
        v.extend(
            iter::repeat(Cuboid::new(record.width, record.depth, record.height)).take(record.count),
        );
    }
    v
}
