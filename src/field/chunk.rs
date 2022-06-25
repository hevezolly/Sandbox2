use crate::elements::Element;

use super::{rect::{Rect, RectIterator}, neighbours::Neighbours, CHUNK_ISIZE};

pub const CHUNK_SIZE: (usize, usize) = (32, 32);

pub type CordInChunk = (usize, usize);

pub struct Chunk{
    field: Vec<Vec<Option<Element>>>,
    field_parity: Vec<Vec<bool>>,
    current_update_area: Rect,
    next_rect: Rect,
    elements_count: usize,
}

impl Chunk {
    pub fn new(parity: bool) -> Chunk{
        let field = vec![vec![None; CHUNK_SIZE.0]; CHUNK_SIZE.1];
        let field_parity = vec![vec![parity; CHUNK_SIZE.0]; CHUNK_SIZE.1];
        Chunk { field, current_update_area: Rect::new(), next_rect: Rect::new(), elements_count: 0, field_parity }
    }

    fn set_value(&mut self, position: CordInChunk, element: Option<Element>) {
        self.field[position.1][position.0] = element;
    }

    pub fn needs_updates(&self) -> bool{
        self.current_update_area.has_value()
    }

    pub fn get(&self, position: CordInChunk) -> Option<Element>{
        self.field[position.1][position.0]
    }

    pub fn parity(&self, position: CordInChunk) -> bool {
        self.field_parity[position.1][position.0]
    }

    pub fn set_parity(&mut self, position: CordInChunk, parity: bool) {
        self.field_parity[position.1][position.0] = parity;
    }

    pub fn set(&mut self, position: CordInChunk, element: Element, parity: bool){
        if self.get(position).is_none(){
            self.elements_count += 1;
            self.next_rect = self.next_rect.expand(position);
        }
        self.set_parity(position, parity);
        self.set_value(position, Some(element));
    }

    pub fn clear(&mut self, position: CordInChunk){
        if self.get(position).is_some(){
            self.elements_count -= 1;
        }
        self.set_value(position, None);
    }

    pub fn add_point_in_update_cycle_with_neighbourhood(&mut self, position: CordInChunk){
        self.next_rect = self.next_rect.expand(position);
        for n in Neighbours::direct_of((position.0 as isize, position.1 as isize)).with_boundaries(Rect::from((0, 0), CHUNK_ISIZE)){
            self.next_rect = self.next_rect.expand((n.0 as usize, n.1 as usize));
        }
    }

    pub fn add_point_in_update_cycle(&mut self, position: CordInChunk){
        self.next_rect = self.next_rect.expand(position);
    }

    pub fn update_rect(&mut self){
        self.current_update_area = self.next_rect;
        self.next_rect = Rect::new();
    }

    pub fn into_iter(&self) -> RectIterator<usize>{
        self.current_update_area.into_iter()
    }

    pub fn get_update_rect(&self) -> Rect{
        self.current_update_area
    }

    pub fn number_of_elements(&self) -> usize{
        self.elements_count
    }
}