use std::collections::BTreeMap;

use crate::point::Point;

#[derive(Debug)]
pub struct Map {
    pub data: BTreeMap<usize, BTreeMap<usize, char>>,
}

impl Map {
    pub fn new() -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    pub fn get(&self, point: &Point) -> Option<char> {
        self.data.get(&(point.y as usize)).and_then(|chars|
                chars.get(&(point.x as usize))).or(Some(&'.')).cloned()
    }

    pub fn set_to_char(&mut self, point: &Point, ch: char) {
        *self.data.entry(point.y as usize).or_insert(BTreeMap::new())
            .entry(point.x as usize).or_insert('.') = ch;
    }

    pub fn empty(&mut self, point: &Point) {
        self.set_to_char(point, '.');
    }
}
