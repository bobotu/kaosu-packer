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

use super::ga::{Chromosome, Decoder as GADecoder};
use super::geom::*;

pub struct Decoder {
    boxes: Vec<InnerBox>,
    bin_spec: Cuboid,
    bin_volume: i32,
    rotation_type: RotationType,
}

impl Decoder {
    pub fn new<'a, T: 'a>(boxes: &'a [T], bin_spec: Cuboid, rotation_type: RotationType) -> Self
    where
        &'a T: Into<Cuboid>,
    {
        let boxes = boxes.iter().map(|b| b.into().into()).collect();
        let bin_volume = bin_spec.volume();
        Decoder {
            boxes,
            bin_spec,
            bin_volume,
            rotation_type,
        }
    }
}

impl GADecoder for Decoder {
    type Solution = InnerSolution;

    fn decode_chromosome(&self, individual: &Chromosome) -> Self::Solution {
        Placer::new(individual, &self.boxes, &self.bin_spec, self.rotation_type).place_boxes()
    }

    fn fitness_of(&self, solution: &Self::Solution) -> f64 {
        solution.num_bins as f64 + (f64::from(solution.least_load) / f64::from(self.bin_volume))
    }
}

struct Placer<'a, 'b> {
    chromosome: &'a Chromosome,
    bin_spec: &'b Cuboid,
    boxes: &'b [InnerBox],
    rotation_type: RotationType,
}

impl<'a, 'b> Placer<'a, 'b> {
    fn new(
        chromosome: &'a Chromosome,
        boxes: &'b [InnerBox],
        bin_spec: &'b Cuboid,
        rotation_type: RotationType,
    ) -> Self {
        Placer {
            chromosome,
            boxes,
            bin_spec,
            rotation_type,
        }
    }

    fn place_boxes(&self) -> InnerSolution {
        let mut placements = Vec::with_capacity(self.boxes.len());
        let mut bins: Vec<(InnerBin, i32)> = Vec::new();
        let bps = self.calculate_bps();

        for (bps_idx, &(box_idx, _)) in bps.iter().enumerate() {
            let box_to_pack = &self.boxes[box_idx];
            let (mut fit_bin, mut fit_space) = (None, None);

            for (i, bin) in bins.iter_mut().enumerate() {
                if let Some(space) = bin
                    .0
                    .find_best_space_for_box(&box_to_pack.rect, self.rotation_type)
                {
                    fit_space = Some(space);
                    fit_bin = Some(i);
                    break;
                }
            }

            if fit_bin.is_none() {
                bins.push((InnerBin::new(*self.bin_spec), 0));
                fit_bin = Some(bins.len() - 1);
                fit_space = Some(&bins[fit_bin.unwrap()].0.empty_space_list[0]);
            }

            let (fit_bin, fit_space) = (fit_bin.unwrap(), fit_space.unwrap());
            let box_placement = self.placement_of_box(box_idx, fit_space);
            let (min_dimension, min_volume) = self.min_dimension_and_volume(&bps[bps_idx..]);

            bins[fit_bin].0.allocate_space(&box_placement, |ns| {
                let (w, d, h) = (ns.width(), ns.depth(), ns.height());
                let v = w * d * h;
                w.min(d).min(h) >= min_dimension && v >= min_volume
            });
            bins[fit_bin].1 += box_to_pack.volume;

            placements.push(InnerPlacement::new(box_placement, fit_bin, box_idx));
        }

        let num_bins = bins.len();
        let least_load = bins.iter().map(|bin| bin.1).min().unwrap();
        InnerSolution::new(num_bins, least_load, placements)
    }

    fn placement_of_box(&self, box_idx: usize, container: &Space) -> Space {
        let rect = &self.boxes[box_idx].rect;
        let gene = self.vbo(box_idx);
        let orientations = self
            .rotation_type
            .orientations_for(rect)
            .into_iter()
            .filter(|rect| rect.can_fit_in(container))
            .collect::<Vec<_>>();
        let decoded_gene = (gene * orientations.len() as f32).ceil() as usize;
        let orientation = &orientations[(decoded_gene).max(1) - 1];
        Space::from_placement(container.origin(), orientation)
    }

    fn min_dimension_and_volume(&self, remain_bps: &[(usize, f32)]) -> (i32, i32) {
        let (mut min_d, mut min_v) = (-1, -1);
        for &(box_idx, _) in remain_bps {
            let b = &self.boxes[box_idx];
            min_d = min_d.min(b.smallest_dimension);
            min_v = min_v.min(b.volume);
        }
        (min_d, min_v)
    }

    #[inline]
    fn calculate_bps(&self) -> Vec<(usize, f32)> {
        let mut pairs = self.chromosome[..self.chromosome.len() / 2]
            .iter()
            .enumerate()
            .map(|(i, &score)| (i, score))
            .collect::<Vec<_>>();
        pairs.sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        pairs
    }

    #[inline]
    fn vbo(&self, i: usize) -> f32 {
        self.chromosome[self.chromosome.len() / 2 + i]
    }
}

struct InnerBin {
    spec: Cuboid,
    empty_space_list: Vec<Space>,
}

impl InnerBin {
    fn new(spec: Cuboid) -> Self {
        let empty_space_list = vec![Space::from_placement(&Point::new(0, 0, 0), &spec)];
        InnerBin {
            spec,
            empty_space_list,
        }
    }

    fn find_best_space_for_box(
        &self,
        rect: &Cuboid,
        rotation_type: RotationType,
    ) -> Option<&Space> {
        let mut max_dist = -1;
        let mut best_ems = None;

        let orientations = rotation_type.orientations_for(rect);
        let container_upper_right = Point::new(self.spec.width, self.spec.depth, self.spec.height);

        for ems in &self.empty_space_list {
            for o in orientations.iter().filter(|o| o.can_fit_in(ems)) {
                let box_upper_right = Space::from_placement(ems.origin(), o).upper_right;
                let dist = container_upper_right.distance2_from(&box_upper_right);
                if dist > max_dist {
                    max_dist = dist;
                    best_ems = Some(ems);
                }
            }
        }

        best_ems
    }

    fn allocate_space<F>(&mut self, space: &Space, mut new_space_filter: F)
    where
        F: FnMut(&Space) -> bool,
    {
        let spaces_intersects = self
            .empty_space_list
            .iter()
            .enumerate()
            .filter(|(_, ems)| ems.intersects(space))
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let new_spaces = spaces_intersects
            .iter()
            .flat_map(|&i| {
                let ems = &self.empty_space_list[i];
                let union = ems.union(space);
                difference_process(&ems, &union, |s| new_space_filter(s))
            })
            .collect::<Vec<_>>();

        for &i in spaces_intersects.iter().rev() {
            self.empty_space_list.swap_remove(i);
        }

        for (i, this) in new_spaces.iter().enumerate() {
            let overlapped = new_spaces
                .iter()
                .enumerate()
                .any(|(j, other)| i != j && other.contains(this));
            if !overlapped {
                self.empty_space_list.push(*this);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InnerPlacement {
    pub space: Space,
    pub bin_no: usize,
    pub box_idx: usize,
}

impl InnerPlacement {
    fn new(space: Space, bin_no: usize, box_idx: usize) -> Self {
        InnerPlacement {
            space,
            bin_no,
            box_idx,
        }
    }
}

#[derive(Debug)]
pub struct InnerBox {
    pub rect: Cuboid,
    pub smallest_dimension: i32,
    pub volume: i32,
}

impl<T> From<T> for InnerBox
where
    T: Into<Cuboid>,
{
    fn from(raw: T) -> Self {
        let rect = raw.into();
        let smallest_dimension = rect.height.min(rect.width).min(rect.depth);
        let volume = rect.volume();
        InnerBox {
            rect,
            smallest_dimension,
            volume,
        }
    }
}

#[derive(Clone, Debug)]
pub struct InnerSolution {
    pub num_bins: usize,
    pub least_load: i32,
    pub placements: Vec<InnerPlacement>,
}

impl InnerSolution {
    fn new(num_bins: usize, least_load: i32, placements: Vec<InnerPlacement>) -> Self {
        InnerSolution {
            num_bins,
            least_load,
            placements,
        }
    }
}

fn difference_process<F>(this: &Space, other: &Space, mut new_space_filter: F) -> Vec<Space>
where
    F: FnMut(&Space) -> bool,
{
    let (sb, su, ob, ou) = (
        &this.bottom_left,
        &this.upper_right,
        &other.bottom_left,
        &other.upper_right,
    );
    [
        Space::new(*sb, Point::new(ob.x, su.y, su.z)),
        Space::new(Point::new(ou.x, sb.y, sb.z), *su),
        Space::new(*sb, Point::new(su.x, ob.y, su.z)),
        Space::new(Point::new(sb.x, ou.y, sb.z), *su),
        Space::new(*sb, Point::new(su.x, su.y, ob.z)),
        Space::new(Point::new(sb.x, sb.y, ou.z), *su),
    ]
    .iter()
    .filter(|ns| ns.width().min(ns.depth()).min(ns.height()) != 0 && new_space_filter(ns))
    .cloned()
    .collect()
}
