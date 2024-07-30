use std::fmt::Debug;
use std::ops::{Index, IndexMut};

/// A point used to index a 2D grid.
#[derive(Clone, Copy)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug)]
pub struct Grid<T> {
    pub array: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl<T: Debug> Grid<T> {
    pub fn new_filled<F>(width: usize, height: usize, setter: F, default: T) -> Self
    where
        F: Fn(Point) -> T,
        T: Copy,
    {
        let mut array = [default].repeat(width * height);

        // use rayon::prelude::*;
        array.iter_mut().enumerate().for_each(|(i, item)| {
            *item = setter(Point {
                x: i % width,
                y: i / width,
            });
        });
        Self {
            array,
            width,
            height,
        }
    }
    pub fn for_each<F>(&mut self, func: F)
    where
        F: Fn(Point, T),
        T: Copy,
    {
        self.array.iter_mut().enumerate().for_each(|(i, item)| {
            func(Point {
                x: i % self.width,
                y: i / self.width,
            }, *item);
        });
    }
}

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self.array[y * self.width + x]
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
            &mut self.array[y * self.width + x]
    }
}
