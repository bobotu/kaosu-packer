#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Point {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Point { x, y, z }
    }

    pub fn distance_between(&self, other: &Self) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    fn scalar_less_than(&self, other: &Self) -> bool {
        self.x <= other.x && self.y <= other.y && self.z <= other.z
    }
}

impl From<(i32, i32, i32)> for Point {
    fn from(tuple: (i32, i32, i32)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub width: i32,
    pub depth: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(width: i32, depth: i32, height: i32) -> Self {
        Rectangle {
            width,
            depth,
            height,
        }
    }

    pub fn orientations(&self) -> Vec<Rectangle> {
        let mut result = Vec::with_capacity(6);

        result.push(Self::new(self.width, self.depth, self.height));
        if self.width != self.depth {
            result.push(Self::new(self.depth, self.width, self.height));
        }

        if self.height != self.depth {
            result.push(Self::new(self.width, self.height, self.depth));
            if self.height != self.width {
                result.push(Self::new(self.height, self.width, self.depth))
            }
        }

        if self.width != self.depth && self.height != self.width {
            result.push(Self::new(self.height, self.depth, self.width));
            if self.height != self.depth {
                result.push(Self::new(self.depth, self.height, self.width));
            }
        }

        result
    }

    pub fn volume(&self) -> i32 {
        self.width * self.depth * self.height
    }

    pub fn can_fit_in(&self, space: &Space) -> bool {
        space.width() >= self.width && space.height() >= self.height && space.depth() >= self.depth
    }
}

impl From<(i32, i32, i32)> for Rectangle {
    fn from(tuple: (i32, i32, i32)) -> Self {
        Self::new(tuple.0, tuple.1, tuple.2)
    }
}

#[derive(Copy, Clone, Debug)]
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

    pub fn from_placement(origin: &Point, rect: &Rectangle) -> Self {
        let x = origin.x + rect.width;
        let y = origin.y + rect.depth;
        let z = origin.z + rect.height;

        Space {
            bottom_left: origin.clone(),
            upper_right: Point::new(x, y, z),
        }
    }

    pub fn origin(&self) -> &Point {
        &self.bottom_left
    }

    pub fn width(&self) -> i32 {
        self.upper_right.x - self.bottom_left.x
    }

    pub fn depth(&self) -> i32 {
        self.upper_right.y - self.bottom_left.y
    }

    pub fn height(&self) -> i32 {
        self.upper_right.z - self.bottom_left.z
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

    pub fn difference_process<F>(&self, other: &Self, mut new_space_filter: F) -> Vec<Self>
    where
        F: FnMut(&Self) -> bool,
    {
        let (sb, su, ob, ou) = (
            &self.bottom_left,
            &self.upper_right,
            &other.bottom_left,
            &other.upper_right,
        );
        [
            Space::new(sb.clone(), (ob.x, su.y, su.z).into()),
            Space::new((ou.x, sb.y, sb.z).into(), su.clone()),
            Space::new(sb.clone(), (su.x, ob.y, su.z).into()),
            Space::new((sb.x, ou.y, sb.z).into(), su.clone()),
            Space::new(sb.clone(), (su.x, su.y, ob.z).into()),
            Space::new((sb.x, sb.y, ou.z).into(), su.clone()),
        ]
        .iter()
        .filter(|ns| ns.width().min(ns.depth()).min(ns.height()) != 0 && new_space_filter(ns))
        .cloned()
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_orientation() {
        assert_eq!(1, Rectangle::new(2, 2, 2).orientations().len());

        assert_eq!(3, Rectangle::new(2, 2, 3).orientations().len());
        assert_eq!(3, Rectangle::new(2, 3, 2).orientations().len());
        assert_eq!(3, Rectangle::new(3, 2, 2).orientations().len());

        assert_eq!(6, Rectangle::new(1, 2, 3).orientations().len());
        assert_eq!(6, Rectangle::new(1, 3, 2).orientations().len());
        assert_eq!(6, Rectangle::new(2, 1, 3).orientations().len());
        assert_eq!(6, Rectangle::new(2, 3, 1).orientations().len());
        assert_eq!(6, Rectangle::new(3, 2, 1).orientations().len());
        assert_eq!(6, Rectangle::new(3, 1, 2).orientations().len());
    }

    #[test]
    fn test_intersects() {
        assert!(space((0, 0, 0), (3, 3, 3)).intersects(&space((1, 1, 1), (2, 2, 2))));
        assert!(space((1, 1, 1), (2, 2, 2)).intersects(&space((0, 0, 0), (3, 3, 3))));
        assert!(space((0, 1, 1), (3, 3, 3)).intersects(&space((1, 0, 0), (2, 4, 2))));

        assert!(!space((0, 0, 0), (3, 3, 3)).intersects(&space((4, 1, 0), (5, 2, 1))));
    }

    fn space(b: (i32, i32, i32), u: (i32, i32, i32)) -> Space {
        Space {
            bottom_left: b.into(),
            upper_right: u.into(),
        }
    }
}
