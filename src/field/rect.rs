
use std::{path::Iter, cmp, num, ops::Add};

use ::num::{One, Zero};

#[derive(Clone, Copy)]
pub struct Rect<T=usize>
where T: Copy+Clone{
    top_left: (T, T),
    bottom_right: (T, T)
}

#[derive(Clone, Copy)]
pub struct RectIterator<T>
where T: Copy + Clone{
    original_rect: Rect<T>,
    current_point: (T, T)
}

impl<T> RectIterator<T>
where T: Copy + Clone{
    pub fn new(original_rect: Rect<T>) -> RectIterator<T>{
        RectIterator { original_rect, current_point: original_rect.top_left }
    }
}

impl<T> Iterator for RectIterator<T> 
where T: PartialEq+Copy+One+Zero+Add+Ord+Clone{
    type Item = (T, T);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.original_rect.has_value(){
            return None;
        }
        if self.current_point.1 >= self.original_rect.bottom_right.1{
            return None;
        }
        let result = self.current_point;
        let mut x = self.current_point.0 + T::one();
        let mut y = self.current_point.1;
        if x >= self.original_rect.bottom_right.0{
            x = self.original_rect.top_left.0;
            y = y + T::one();
        }
        self.current_point = (x, y);

        Some(result)
    }
}

impl Rect<isize> {
    pub fn from_center(center: (isize, isize), size: (usize, usize)) -> Rect<isize>{
        let top = (center.1 - size.1 as isize /2);

        let left = (center.0 - size.0 as isize /2);

        let bottom = (center.1 + size.1 as isize / 2 +size.1 as isize % 2);

        let right = (center.0 + size.0 as isize / 2 +size.0 as isize % 2);

        Rect::from((left, top), (right, bottom))
    }
}

impl<T> Rect<T>
where T: PartialEq+Copy+One+Zero+Add+Ord+Clone{

    pub fn new() -> Rect<T> {Rect { top_left: (T::zero(),T::zero()), bottom_right: (T::zero(),T::zero()) }}

    pub fn from(top_left: (T, T), bottom_right: (T, T)) -> Rect<T>{
        Rect { top_left, bottom_right }
    }

    pub fn left(&self) -> T {self.top_left.0}
    pub fn right(&self) -> T {self.bottom_right.0}
    pub fn top(&self) -> T {self.top_left.1}
    pub fn bottom(&self) -> T {self.bottom_right.1}

    pub fn has_value(&self) -> bool {self.bottom_right.0 != self.top_left.0 || self.bottom_right.1 != self.top_left.1}

    pub fn into_iter(&self) -> RectIterator<T> {RectIterator::new(*self)}

    pub fn is_inside(&self, point: (T,T)) -> bool {
        point.0 >= self.top_left.0 && point.1 >= self.top_left.1 && point.1 < self.bottom_right.1 && point.0 < self.bottom_right.0 
    }

    pub fn is_inside_inclusive(&self, point: (T,T)) -> bool {
        point.0 >= self.top_left.0 && point.1 >= self.top_left.1 && point.1 <= self.bottom_right.1 && point.0 <= self.bottom_right.0 
    }

    pub fn expand(&self, point: (T, T)) -> Rect<T> {
        if !self.has_value(){
            return Rect{top_left: point, bottom_right: (point.0 + T::one(), point.1 + T::one())};
        }
        let left = cmp::min(point.0, self.left());
        let right = cmp::max(point.0 + T::one(), self.right());
        let top = cmp::min(point.1, self.top());
        let bottom = cmp::max(point.1 + T::one(), self.bottom());
        Rect { top_left: (left, top), bottom_right: (right, bottom) }
    }
}
