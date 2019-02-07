use super::ga::Chromosome;
use super::geometry::*;
use super::inner::*;

pub struct Placer<'a, 'b> {
    chromosome: &'a Chromosome,
    bin_spec: &'b Rectangle,
    boxes: &'b [InnerBox],
    rotation_type: RotationType,
}

impl<'a, 'b> Placer<'a, 'b> {
    pub fn new(
        chromosome: &'a Chromosome,
        boxes: &'b [InnerBox],
        bin_spec: &'b Rectangle,
        rotation_type: RotationType,
    ) -> Self {
        Placer {
            chromosome,
            boxes,
            bin_spec,
            rotation_type,
        }
    }

    pub fn place_boxes(&self) -> InnerSolution {
        let mut placements = Vec::with_capacity(self.boxes.len());
        let mut bins: Vec<(Bin, i32)> = Vec::new();
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
                bins.push((Bin::new(*self.bin_spec), 0));
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
        let orientations = rect
            .orientations(self.rotation_type)
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

struct Bin {
    spec: Rectangle,
    empty_space_list: Vec<Space>,
}

impl Bin {
    fn new(spec: Rectangle) -> Self {
        let empty_space_list = vec![Space::from_placement(&(0, 0, 0).into(), &spec)];
        Bin {
            spec,
            empty_space_list,
        }
    }

    fn find_best_space_for_box(
        &self,
        rect: &Rectangle,
        rotation_type: RotationType,
    ) -> Option<&Space> {
        let mut max_dist = -1;
        let mut best_ems = None;

        let orientations = rect.orientations(rotation_type);
        let container_upper_right = Point::new(self.spec.width, self.spec.depth, self.spec.height);

        for ems in &self.empty_space_list {
            for o in orientations.iter().filter(|o| o.can_fit_in(ems)) {
                let box_upper_right = Space::from_placement(ems.origin(), o).upper_right;
                let dist = container_upper_right.distance_between(&box_upper_right);
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
                ems.difference_process(&union, |s| new_space_filter(s))
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
