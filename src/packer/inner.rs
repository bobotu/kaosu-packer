use super::geometry::*;

#[derive(Clone, Debug)]
pub struct InnerPlacement {
    pub space: Space,
    pub bin_no: usize,
    pub box_idx: usize,
}

impl InnerPlacement {
    pub fn new(space: Space, bin_no: usize, box_idx: usize) -> Self {
        InnerPlacement {
            space,
            bin_no,
            box_idx,
        }
    }
}

#[derive(Debug)]
pub struct InnerBox {
    pub rect: Rectangle,
    pub smallest_dimension: i32,
    pub volume: i32,
}

impl<T> From<T> for InnerBox
where
    T: Into<Rectangle>,
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
    pub fn new(num_bins: usize, least_load: i32, placements: Vec<InnerPlacement>) -> Self {
        InnerSolution {
            num_bins,
            least_load,
            placements,
        }
    }
}
