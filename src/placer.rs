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

use std::cell::RefCell;
use std::i32;

use super::ga::{Chromosome, Decoder as GADecoder};
use super::geom::*;

pub struct Decoder {
    bin_volume: i32,
    placer: Placer,
}

impl Decoder {
    pub fn new<'a, T: 'a>(boxes: &'a [T], bin_spec: Cuboid, rotation_type: RotationType) -> Self
    where
        &'a T: Into<Cuboid>,
    {
        let boxes = boxes.iter().map(|b| b.into().into()).collect();
        let bin_volume = bin_spec.volume();
        let placer = Placer::new(boxes, bin_spec, rotation_type);
        Decoder { placer, bin_volume }
    }
}

impl GADecoder for Decoder {
    type Solution = InnerSolution;

    fn decode_chromosome(&mut self, individual: &Chromosome) -> Self::Solution {
        self.placer.place_boxes(individual)
    }

    fn fitness_of(&self, solution: &Self::Solution) -> f64 {
        solution.num_bins as f64 + (f64::from(solution.least_load) / f64::from(self.bin_volume))
    }

    fn reset(&mut self) {
        self.placer.reset();
    }
}

struct Placer {
    boxes: Vec<InnerBox>,
    rotation_type: RotationType,

    bins: BinList,
    bps: Vec<(usize, f32)>,
    orientations: RefCell<Vec<Cuboid>>,
}

impl Placer {
    fn new(boxes: Vec<InnerBox>, bin_spec: Cuboid, rotation_type: RotationType) -> Self {
        Placer {
            boxes,
            rotation_type,
            bins: BinList::new(bin_spec),
            bps: Vec::new(),
            orientations: RefCell::new(Vec::new()),
        }
    }

    fn place_boxes(&mut self, chromosome: &Chromosome) -> InnerSolution {
        let mut placements = Vec::with_capacity(self.boxes.len());
        let (mut min_dimension, mut min_volume) = (i32::MAX, i32::MAX);

        self.calculate_bps(chromosome);
        for (bps_idx, &(box_idx, _)) in self.bps.iter().enumerate() {
            let box_to_pack = &self.boxes[box_idx];
            let (mut fit_bin, mut fit_space) = (None, None);

            for (i, bin) in self.bins.opened().iter().enumerate() {
                let placement = bin.try_place_cuboid(&box_to_pack.cuboid, self.rotation_type);
                if let Some(space) = placement {
                    fit_space = Some(space);
                    fit_bin = Some(i);
                    break;
                }
            }

            if fit_bin.is_none() {
                let idx = self.bins.open_new_bin();
                fit_bin = Some(idx);
                fit_space = Some(&self.bins.nth(idx).empty_space_list[0]);
            }

            let (fit_bin, fit_space) = (fit_bin.unwrap(), fit_space.unwrap());
            let placement = self.place_box(box_idx, chromosome, fit_space);

            if box_to_pack.smallest_dimension <= min_dimension || box_to_pack.volume <= min_volume {
                let (md, mv) = self.min_dimension_and_volume(&self.bps[bps_idx + 1..]);
                min_dimension = md;
                min_volume = mv;
            }

            self.bins.nth_mut(fit_bin).allocate_space(&placement, |ns| {
                let (w, d, h) = (ns.width(), ns.depth(), ns.height());
                let v = w * d * h;
                w.min(d).min(h) >= min_dimension && v >= min_volume
            });

            placements.push(InnerPlacement::new(placement, fit_bin, box_idx));
        }

        let bins = self.bins.opened();
        let num_bins = bins.len();
        let least_load = bins.iter().map(|bin| bin.used_volume).min().unwrap();
        InnerSolution::new(num_bins, least_load, placements)
    }

    fn place_box(&self, box_idx: usize, chromosome: &Chromosome, container: &Space) -> Space {
        let cuboid = &self.boxes[box_idx].cuboid;
        let gene = chromosome[chromosome.len() / 2 + box_idx];

        let mut orientations = self.orientations.borrow_mut();
        orientations.clear();
        rotate_cuboid(self.rotation_type, cuboid, orientations.as_mut());
        orientations.retain(|c| c.can_fit_in(container));

        let decoded_gene = (gene * orientations.len() as f32).ceil() as usize;
        let orientation = &orientations[(decoded_gene).max(1) - 1];
        Space::from_placement(container.origin(), orientation)
    }

    fn reset(&mut self) {
        self.bins.reset();
        self.bps.clear();
        self.orientations.borrow_mut().clear();
    }

    fn min_dimension_and_volume(&self, remain_bps: &[(usize, f32)]) -> (i32, i32) {
        let (mut min_d, mut min_v) = (i32::MAX, i32::MAX);
        for &(box_idx, _) in remain_bps {
            let b = &self.boxes[box_idx];
            min_d = min_d.min(b.smallest_dimension);
            min_v = min_v.min(b.volume);
        }
        (min_d, min_v)
    }

    #[inline]
    fn calculate_bps(&mut self, chromosome: &Chromosome) {
        self.bps.clear();
        let bps = chromosome[..chromosome.len() / 2]
            .iter()
            .enumerate()
            .map(|(i, &score)| (i, score));
        self.bps.extend(bps);

        self.bps
            .sort_unstable_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    }
}

struct BinList {
    spec: Cuboid,
    bins: Vec<InnerBin>,
    size: usize,
}

impl BinList {
    fn new(spec: Cuboid) -> Self {
        BinList {
            spec,
            bins: Vec::new(),
            size: 0,
        }
    }

    fn nth_mut(&mut self, idx: usize) -> &mut InnerBin {
        &mut self.bins[idx]
    }

    fn nth(&self, idx: usize) -> &InnerBin {
        &self.bins[idx]
    }

    fn opened(&self) -> &[InnerBin] {
        &self.bins[0..self.size]
    }

    fn open_new_bin(&mut self) -> usize {
        let buffered = self.bins.len() - self.size;
        if buffered == 0 {
            self.bins.push(InnerBin::new(self.spec));
        } else {
            self.bins[self.size].reset();
        }
        self.size += 1;
        self.size - 1
    }

    fn reset(&mut self) {
        self.size = 0
    }
}

struct InnerBin {
    spec: Cuboid,
    used_volume: i32,

    empty_space_list: Vec<Space>,
    spaces_intersects: Vec<usize>,
    new_empty_spaces: Vec<Space>,
    orientations: RefCell<Vec<Cuboid>>,
}

impl InnerBin {
    fn new(spec: Cuboid) -> Self {
        let empty_space_list = vec![Space::from_placement(&Point::new(0, 0, 0), &spec)];
        InnerBin {
            spec,
            empty_space_list,
            used_volume: 0,
            spaces_intersects: Vec::new(),
            new_empty_spaces: Vec::new(),
            orientations: RefCell::new(Vec::with_capacity(6)),
        }
    }

    fn try_place_cuboid(&self, cuboid: &Cuboid, rotation_type: RotationType) -> Option<&Space> {
        let mut max_dist = -1;
        let mut best_ems = None;
        let mut orientations = self.orientations.borrow_mut();

        orientations.clear();
        rotate_cuboid(rotation_type, cuboid, orientations.as_mut());
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
        self.used_volume += space.volume();

        self.spaces_intersects.clear();
        let spaces_intersects = self
            .empty_space_list
            .iter()
            .enumerate()
            .filter(|(_, ems)| ems.intersects(space))
            .map(|(i, _)| i);
        self.spaces_intersects.extend(spaces_intersects);

        self.new_empty_spaces.clear();
        for &i in self.spaces_intersects.iter() {
            let ems = &self.empty_space_list[i];
            let union = ems.union(space);
            difference_process(ems, &union, &mut self.new_empty_spaces, |s| {
                new_space_filter(s)
            })
        }

        for &i in self.spaces_intersects.iter().rev() {
            self.empty_space_list.swap_remove(i);
        }
        self.empty_space_list.retain(|s| new_space_filter(s));

        for (i, this) in self.new_empty_spaces.iter().enumerate() {
            let overlapped = self
                .new_empty_spaces
                .iter()
                .enumerate()
                .any(|(j, other)| i != j && other.contains(this));
            if !overlapped {
                self.empty_space_list.push(*this);
            }
        }
    }

    #[inline]
    fn reset(&mut self) {
        self.used_volume = 0;
        self.orientations.borrow_mut().clear();
        self.new_empty_spaces.clear();
        self.spaces_intersects.clear();
        self.empty_space_list.clear();
        self.empty_space_list
            .push(Space::from_placement(&Point::new(0, 0, 0), &self.spec))
    }
}

#[inline]
fn difference_process<F>(
    this: &Space,
    other: &Space,
    new_spaces: &mut Vec<Space>,
    mut new_space_filter: F,
) where
    F: FnMut(&Space) -> bool,
{
    let (sb, su, ob, ou) = (
        &this.bottom_left,
        &this.upper_right,
        &other.bottom_left,
        &other.upper_right,
    );
    let spaces = [
        Space::new(*sb, Point::new(ob.x, su.y, su.z)),
        Space::new(Point::new(ou.x, sb.y, sb.z), *su),
        Space::new(*sb, Point::new(su.x, ob.y, su.z)),
        Space::new(Point::new(sb.x, ou.y, sb.z), *su),
        Space::new(*sb, Point::new(su.x, su.y, ob.z)),
        Space::new(Point::new(sb.x, sb.y, ou.z), *su),
    ];

    let spaces = spaces
        .iter()
        .filter(|ns| ns.width().min(ns.depth()).min(ns.height()) != 0 && new_space_filter(ns));
    for space in spaces {
        new_spaces.push(*space);
    }
}

fn rotate_cuboid(tp: RotationType, cuboid: &Cuboid, orientations: &mut Vec<Cuboid>) {
    let only_2d = match tp {
        RotationType::TwoDimension => true,
        RotationType::ThreeDimension => false,
    };

    orientations.push(Cuboid::new(cuboid.width, cuboid.depth, cuboid.height));
    if cuboid.width != cuboid.depth {
        orientations.push(Cuboid::new(cuboid.depth, cuboid.width, cuboid.height));
    }

    if !only_2d {
        if cuboid.height != cuboid.depth {
            orientations.push(Cuboid::new(cuboid.width, cuboid.height, cuboid.depth));
            if cuboid.height != cuboid.width {
                orientations.push(Cuboid::new(cuboid.height, cuboid.width, cuboid.depth))
            }
        }

        if cuboid.width != cuboid.depth && cuboid.height != cuboid.width {
            orientations.push(Cuboid::new(cuboid.height, cuboid.depth, cuboid.width));
            if cuboid.height != cuboid.depth {
                orientations.push(Cuboid::new(cuboid.depth, cuboid.height, cuboid.width));
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
    pub cuboid: Cuboid,
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
            cuboid: rect,
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
