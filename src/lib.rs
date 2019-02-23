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

pub mod geom;

mod ga;
mod placer;

#[cfg(feature = "serde")]
use serde::*;

use self::ga::{RandGenerator, Solver};
use self::geom::{Cuboid, RotationType, Space};
use self::placer::Decoder;

#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Placement {
    pub space: Space,
    pub item_idx: usize,
}

pub type PackSolution = Vec<Vec<Placement>>;

macro_rules! do_pack {
    ($params:ident, $bin_spec:ident, $boxes:ident) => {{
        let generator = RandGenerator::new($boxes.len() * 2);
        let ga_params = $params.get_ga_params($boxes.len());
        let mut solver = Solver::new(ga_params, generator, || {
            Decoder::new($boxes, $bin_spec, $params.box_rotation_type)
        });
        let solution = solver.solve();

        let mut bins = vec![Vec::new(); solution.num_bins];
        for inner_placement in &solution.placements {
            let idx = inner_placement.bin_no;
            let space = inner_placement.space;
            let item_idx = inner_placement.box_idx;
            bins[idx].push(Placement { space, item_idx })
        }
        bins
    }};
}

#[cfg(feature = "rayon")]
pub fn pack_boxes<'a, T>(params: Params, bin_spec: Cuboid, boxes: &'a [T]) -> PackSolution
where
    T: Sync,
    &'a T: Into<Cuboid>,
{
    do_pack!(params, bin_spec, boxes)
}

#[cfg(not(feature = "rayon"))]
pub fn pack_boxes<'a, T>(params: Params, bin_spec: Cuboid, boxes: &'a [T]) -> PackSolution
where
    &'a T: Into<Cuboid>,
{
    do_pack!(params, bin_spec, boxes)
}
