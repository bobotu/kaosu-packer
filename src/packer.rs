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

mod ga;
mod geometry;
mod inner;
mod placer;

use self::ga::{Chromosome, RandGenerator, Solver};
use self::geometry::*;
use self::inner::*;
use self::placer::Placer;

pub type GAParams = ga::Params;
pub use self::geometry::RotationType;

#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub ga_params: GAParams,
    pub box_rotation_type: RotationType,
}

impl Params {
    pub fn new(ga_params: GAParams, box_rotation_type: RotationType) -> Self {
        Params {
            ga_params,
            box_rotation_type,
        }
    }
}

pub fn pack_boxes<'a, T>(
    params: Params,
    bin_spec: Dimension,
    boxes: &'a [T],
) -> Vec<Vec<Placement<'a, T>>>
where
    &'a T: Into<Dimension>,
{
    let decoder = Decoder::new(boxes, bin_spec, params.box_rotation_type);
    let generator = RandGenerator::new(boxes.len() * 2);
    let mut solver = Solver::new(params.ga_params, generator, decoder);
    let solution = solver.solve();

    let mut bins = vec![Vec::new(); solution.num_bins];
    for inner_placement in &solution.placements {
        let idx = inner_placement.bin_no;
        let space = inner_placement.space;
        let item = &boxes[inner_placement.box_idx];
        bins[idx].push(Placement { space, item })
    }
    bins
}

pub fn recommend_ga_params(problem_size: usize) -> GAParams {
    let population_size = 30 * problem_size;
    GAParams {
        population_size,
        num_elites: (0.10 * population_size as f64) as usize,
        num_mutants: (0.15 * population_size as f64) as usize,
        inherit_elite_probability: 0.70,
        max_generations: 200,
        max_generations_no_improvement: 5,
    }
}

pub struct Placement<'a, T> {
    pub space: Space,
    pub item: &'a T,
}

impl<T> Clone for Placement<'_, T> {
    fn clone(&self) -> Self {
        Placement {
            space: self.space,
            item: self.item,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Dimension {
    pub width: i32,
    pub depth: i32,
    pub height: i32,
}

impl Dimension {
    pub fn new(width: i32, depth: i32, height: i32) -> Self {
        Dimension {
            width,
            depth,
            height,
        }
    }
}

impl From<&(i32, i32, i32)> for Dimension {
    fn from(t: &(i32, i32, i32)) -> Self {
        Self::new(t.0, t.1, t.2)
    }
}

impl Into<Rectangle> for Dimension {
    fn into(self) -> Rectangle {
        Rectangle::new(self.width, self.depth, self.height)
    }
}

struct Decoder {
    boxes: Vec<InnerBox>,
    bin_spec: Rectangle,
    bin_volume: i32,
    rotation_type: RotationType,
}

impl Decoder {
    fn new<'a, T: 'a>(boxes: &'a [T], bin_spec: Dimension, rotation_type: RotationType) -> Self
    where
        &'a T: Into<Dimension>,
    {
        let boxes = boxes.iter().map(|b| b.into().into()).collect();
        let bin_spec: Rectangle = bin_spec.into();
        let bin_volume = bin_spec.volume();
        Decoder {
            boxes,
            bin_spec,
            bin_volume,
            rotation_type,
        }
    }
}

impl ga::Decoder for Decoder {
    type Solution = InnerSolution;

    fn decode_chromosome(&self, individual: &Chromosome) -> Self::Solution {
        Placer::new(individual, &self.boxes, &self.bin_spec, self.rotation_type).place_boxes()
    }

    fn fitness_of(&self, solution: &Self::Solution) -> f64 {
        solution.num_bins as f64 + (f64::from(solution.least_load) / f64::from(self.bin_volume))
    }
}
