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

#[cfg(feature = "serde")]
use serde::*;

#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Point {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Point { x, y, z }
    }

    pub fn distance2_from(&self, other: &Self) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    fn scalar_less_than(&self, other: &Self) -> bool {
        self.x <= other.x && self.y <= other.y && self.z <= other.z
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Cuboid {
    pub width: i32,
    pub depth: i32,
    pub height: i32,
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RotationType {
    ThreeDimension,
    TwoDimension,
}

impl RotationType {
    pub fn orientations_for(self, rect: &Cuboid) -> Vec<Cuboid> {
        let only_2d = match self {
            RotationType::TwoDimension => true,
            RotationType::ThreeDimension => false,
        };
        let mut result = Vec::with_capacity(if only_2d { 2 } else { 6 });

        result.push(Cuboid::new(rect.width, rect.depth, rect.height));
        if rect.width != rect.depth {
            result.push(Cuboid::new(rect.depth, rect.width, rect.height));
        }

        if !only_2d {
            if rect.height != rect.depth {
                result.push(Cuboid::new(rect.width, rect.height, rect.depth));
                if rect.height != rect.width {
                    result.push(Cuboid::new(rect.height, rect.width, rect.depth))
                }
            }

            if rect.width != rect.depth && rect.height != rect.width {
                result.push(Cuboid::new(rect.height, rect.depth, rect.width));
                if rect.height != rect.depth {
                    result.push(Cuboid::new(rect.depth, rect.height, rect.width));
                }
            }
        }

        result
    }
}

impl Cuboid {
    pub fn new(width: i32, depth: i32, height: i32) -> Self {
        Cuboid {
            width,
            depth,
            height,
        }
    }

    pub fn volume(&self) -> i32 {
        self.width * self.depth * self.height
    }

    pub fn can_fit_in(&self, space: &Space) -> bool {
        space.width() >= self.width && space.height() >= self.height && space.depth() >= self.depth
    }
}

impl Into<Cuboid> for &Cuboid {
    fn into(self) -> Cuboid {
        *self
    }
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Space {
    pub bottom_left: Point,
    pub upper_right: Point,
}

impl Space {
    pub fn new(bottom_left: Point, upper_right: Point) -> Self {
        Space {
            bottom_left,
            upper_right,
        }
    }

    pub fn from_placement(origin: &Point, rect: &Cuboid) -> Self {
        let x = origin.x + rect.width;
        let y = origin.y + rect.height;
        let z = origin.z + rect.depth;

        Space {
            bottom_left: *origin,
            upper_right: Point::new(x, y, z),
        }
    }

    pub fn origin(&self) -> &Point {
        &self.bottom_left
    }

    pub fn center(&self) -> (f64, f64, f64) {
        let x = (f64::from(self.upper_right.x) + f64::from(self.bottom_left.x)) / 2.;
        let y = (f64::from(self.upper_right.y) + f64::from(self.bottom_left.y)) / 2.;
        let z = (f64::from(self.upper_right.z) + f64::from(self.bottom_left.z)) / 2.;
        (x, y, z)
    }

    pub fn width(&self) -> i32 {
        self.upper_right.x - self.bottom_left.x
    }

    pub fn depth(&self) -> i32 {
        self.upper_right.z - self.bottom_left.z
    }

    pub fn height(&self) -> i32 {
        self.upper_right.y - self.bottom_left.y
    }

    pub fn contains(&self, other: &Self) -> bool {
        self.bottom_left.scalar_less_than(&other.bottom_left)
            && other.upper_right.scalar_less_than(&self.upper_right)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.bottom_left.scalar_less_than(&other.upper_right)
            && other.bottom_left.scalar_less_than(&self.upper_right)
    }

    pub fn union(&self, other: &Self) -> Self {
        let bx = self.bottom_left.x.max(other.bottom_left.x);
        let by = self.bottom_left.y.max(other.bottom_left.y);
        let bz = self.bottom_left.z.max(other.bottom_left.z);
        let ux = self.upper_right.x.min(other.upper_right.x);
        let uy = self.upper_right.y.min(other.upper_right.y);
        let uz = self.upper_right.z.min(other.upper_right.z);

        Space::new(Point::new(bx, by, bz), Point::new(ux, uy, uz))
    }
}
