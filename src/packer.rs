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

use serde::*;

use self::ga::{Chromosome, RandGenerator, Solver};
use self::geometry::*;
use self::inner::*;
use self::placer::Placer;

pub use self::geometry::{Point, RotationType, Space};

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Params {
    pub population_factor: usize,
    pub elites_percentage: f64,
    pub mutants_percentage: f64,
    pub inherit_elite_probability: f64,
    pub max_generations: i32,
    pub max_generations_no_improvement: i32,
    pub box_rotation_type: RotationType,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            population_factor: 30,
            elites_percentage: 0.10,
            mutants_percentage: 0.15,
            inherit_elite_probability: 0.70,
            max_generations: 200,
            max_generations_no_improvement: 5,
            box_rotation_type: RotationType::ThreeDimension,
        }
    }
}

impl Params {
    fn get_ga_params(&self, num_items: usize) -> ga::Params {
        let population_size = self.population_factor * num_items;
        let num_elites = (self.elites_percentage * population_size as f64) as usize;
        let num_mutants = (self.mutants_percentage * population_size as f64) as usize;
        ga::Params {
            population_size,
            num_elites,
            num_mutants,
            inherit_elite_probability: self.inherit_elite_probability,
            max_generations: self.max_generations,
            max_generations_no_improvement: self.max_generations_no_improvement,
        }
    }
}

pub fn pack_boxes<'a, T>(
    params: Params,
    bin_spec: Dimension,
    boxes: &'a [T],
) -> Vec<Vec<Placement<T>>>
where
    &'a T: Into<Dimension>,
    T: Clone,
{
    let decoder = Decoder::new(boxes, bin_spec, params.box_rotation_type);
    let generator = RandGenerator::new(boxes.len() * 2);
    let ga_params = params.get_ga_params(boxes.len());
    let mut solver = Solver::new(ga_params, generator, decoder);
    let solution = solver.solve();

    let mut bins = vec![Vec::new(); solution.num_bins];
    for inner_placement in &solution.placements {
        let idx = inner_placement.bin_no;
        let space = inner_placement.space;
        let item = boxes[inner_placement.box_idx].clone();
        bins[idx].push(Placement { space, item })
    }
    bins
}

#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub struct Placement<T> {
    pub space: Space,
    pub item: T,
}

#[derive(PartialEq, Default, Serialize, Deserialize, Copy, Clone, Debug)]
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
