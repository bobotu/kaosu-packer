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
