use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid<T> {
    pub array: Vec<T>,
    pub width: usize,
    pub height: usize,
}

impl<T: Debug> Grid<T> {
    pub fn new_filled<F>(width: usize, height: usize, setter: F, default: T) -> Self
    where
        F: Fn(UVec2) -> T,
        T: Copy,
    {
        let mut array = [default].repeat(width * height);

        // use rayon::prelude::*;
        array.iter_mut().enumerate().for_each(|(i, item)| {
            *item = setter(uvec2(
                (i % width) as u32,
                (i / width) as u32,
            ));
        });
        Self {
            array,
            width,
            height,
        }
    }
    pub fn for_each<F>(&mut self, func: F)
    where
        F: Fn(UVec2, T),
        T: Copy,
    {
        self.array.iter_mut().enumerate().for_each(|(i, item)| {
            func(uvec2(
                (i % self.width) as u32,
                (i / self.width) as u32,
            ), *item);
        });
    }
    pub fn for_each_immut<F>(&self, func: F)
    where
        F: Fn(UVec2, T),
        T: Copy,
    {
        self.array.iter().enumerate().for_each(|(i, item)| {
            func(uvec2(
                (i % self.width) as u32,
                (i / self.width) as u32,
            ), *item);
        });
    }
    
    pub fn get(&self, v: UVec2) -> Option<&T> {
        self.array.get((v.y * self.width as u32 + v.x) as usize)
    }
    pub fn get_mut(&mut self, v: UVec2) -> Option<&mut T> {
        self.array.get_mut((v.y * self.width as u32 + v.x) as usize)
    }


}

impl<T> Index<UVec2> for Grid<T> {
    type Output = T;

    fn index(&self, v: UVec2) -> &Self::Output {
        &self.array[(v.y * self.width as u32 + v.x) as usize]
    }
}

impl<T> IndexMut<UVec2> for Grid<T> {
    fn index_mut(&mut self, v: UVec2) -> &mut Self::Output {
        &mut self.array[(v.y * self.width as u32 + v.x) as usize]
    }
}
