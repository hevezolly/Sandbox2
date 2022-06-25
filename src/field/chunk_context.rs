use std::{cell::RefCell, collections::HashMap, rc::Rc, ops::Deref, sync::{Mutex, MutexGuard, Arc, RwLock}, hash::Hash};

use crate::elements::{Element, ElementData};

use super::{chunk::{Chunk, self, CordInChunk}, ChunkCord, global_cord_to_chunk_local, neighbours::Neighbours, ChunkRef};

#[derive(Clone, Copy)]
pub enum UnsolvedActions{
    MissingChunkInsertion{
        chunk_cord: ChunkCord,
        insertion_cord: CordInChunk,
        elementToInsert: Element
    }
}


pub struct ChunkContext{
    current_chunk: ChunkRef,
    current_chunk_cord: ChunkCord, 
    neighbours: HashMap<ChunkCord, Option<ChunkRef>>,
    pub unsolved_actions: Vec<UnsolvedActions>,
    pub updated_coordinates: Vec<(isize, isize)>,
    parity: bool
}

impl ChunkContext {

    pub fn new(current_chunk: ChunkRef,
    current_chunk_cord: ChunkCord,
    neighbours: HashMap<ChunkCord, Option<ChunkRef>>, parity: bool) -> ChunkContext{
        ChunkContext { current_chunk, current_chunk_cord, neighbours, 
            unsolved_actions: Vec::new(), parity, updated_coordinates: Vec::new() }
    }

    fn is_in_neighbour_range(&self, cord: ChunkCord) -> bool{
        (cord.0 - self.current_chunk_cord.0).abs() <= 1 && (cord.1 - self.current_chunk_cord.1).abs() <= 1
    }

    pub fn current_chunk(&self) -> &ChunkRef{
        &self.current_chunk
    }

    pub fn get(&self, position: (isize, isize)) -> Result<Option<Element>, ()>{
        let (chunk_c, in_chunk_c) = global_cord_to_chunk_local(position);
        if !self.is_in_neighbour_range(chunk_c){
            return Err(());
        }
        if self.current_chunk_cord == chunk_c{
            return Ok(self.current_chunk.read().unwrap().get(in_chunk_c));
        }
        let chunk_access = self.neighbours.get(&chunk_c);
        if let Some(chunk_option) = chunk_access{
            if let Some(chunk) = chunk_option{
                return Ok(chunk.read().unwrap().get(in_chunk_c));
            }
            return Ok(None);
        } 
        Err(())
    }

    fn keep_adjesent_cells_alive(&mut self, position: (isize, isize)){
        for neighbour in Neighbours::of(position){
            let (chunk_c, in_chunk_c) = global_cord_to_chunk_local(neighbour);

            if self.current_chunk_cord == chunk_c{
                let mut lock = self.current_chunk.write().unwrap();
                lock.add_point_in_update_cycle(in_chunk_c);
                if let Some(element) = lock.get(in_chunk_c){
                    let parity = lock.parity(in_chunk_c);
                    lock.set(in_chunk_c, element.refresh(), parity);
                }
            }
            else if let Some(Some(chunk)) = self.neighbours.get(&chunk_c){
                let mut lock = chunk.write().unwrap();
                lock.add_point_in_update_cycle(in_chunk_c);
                if let Some(element) = lock.get(in_chunk_c){
                    let parity = lock.parity(in_chunk_c);
                    lock.set(in_chunk_c, element.refresh(), parity);
                }
            }
        }
    }

    pub fn reachable_and_fitting(&self, position: (isize, isize), fit_func: impl Fn(Option<Element>) -> bool) -> bool{
        let result = self.get(position);
        if let Err(_) = result{
            return false;
        } 
        return fit_func(result.unwrap())
    }

    pub fn reachable_empty_or_fitting(&self, position: (isize, isize), fit_func: impl Fn(Element) -> bool) -> bool{
        let result = self.get(position);
        if let Ok(None) = result{
            return true;
        } 
        else if let Ok(Some(element)) = result{
            return fit_func(element);
        }
        false
    }

    pub fn empty_and_reachable(&self, position: (isize, isize)) -> bool {
        let result = self.get(position);
        if let Ok(None) = result{
            return true;
        } 
        false
    }

    pub fn keep_alive_local(&mut self, position: CordInChunk){
        self.current_chunk.write().unwrap().add_point_in_update_cycle(position);
    }

    pub fn keep_alive(&mut self, position: (isize, isize)){
        let (chunk_c, in_chunk_c) = global_cord_to_chunk_local(position);
        if !self.is_in_neighbour_range(chunk_c){
            return;
        }
        if self.current_chunk_cord == chunk_c{
            self.keep_alive_local(in_chunk_c);
        }
        else if let Some(Some(chunk)) = self.neighbours.get(&chunk_c){
            let mut lock = chunk.write().unwrap();
            lock.add_point_in_update_cycle(in_chunk_c);
        }
    }

    pub fn clear(&mut self, position: (isize, isize)){
        let (chunk_c, in_chunk_c) = global_cord_to_chunk_local(position);
        if !self.is_in_neighbour_range(chunk_c){
            return;
        }

        if self.current_chunk_cord == chunk_c{
            self.current_chunk.write().unwrap().clear(in_chunk_c);
        }
        else if let Some(Some(chunk)) = self.neighbours.get(&chunk_c){
            let mut lock = chunk.write().unwrap();
            lock.clear(in_chunk_c);
        }
        self.updated_coordinates.push(position);
        self.keep_adjesent_cells_alive(position);
    }

    pub fn set(&mut self, position: (isize, isize), element: Element) {
        self.set_internal(position, element, true);
    }

    pub fn set_static(&mut self, position: (isize, isize), element: Element) {
        self.set_internal(position, element, false);
    }

    pub fn move_from_to(&mut self, from: (isize, isize), to: (isize, isize), element: Element){
        let other = self.get(to).unwrap();
        self.set(to, element);
        match other {
            Some(element) => self.set(from, element),
            None => self.clear(from),
        }
    }

    fn set_internal(&mut self, position: (isize, isize), element: Element, keep_adjesent_alive: bool){
        let (chunk_c, in_chunk_c) = global_cord_to_chunk_local(position);
        if !self.is_in_neighbour_range(chunk_c){
            return;
        }
        {
            if self.current_chunk_cord == chunk_c{
                self.current_chunk.write().unwrap().set(in_chunk_c, element, !self.parity);
            }
            else if let Some(Some(chunk)) = self.neighbours.get(&chunk_c){
                let mut lock = chunk.write().unwrap();
                lock.set(in_chunk_c, element, !self.parity);
            }
            else {
                if let Some(chunk) = self.neighbours.get(&chunk_c){
                    self.unsolved_actions.push(UnsolvedActions::MissingChunkInsertion 
                        { chunk_cord: chunk_c, insertion_cord: in_chunk_c, elementToInsert: element });
                }
                return;
            }
        }
        self.updated_coordinates.push(position);
        if keep_adjesent_alive{
            self.keep_adjesent_cells_alive(position);
        }
    }

    pub fn current_chunk_cord(&self) -> (isize, isize) {
        self.current_chunk_cord
    }

    pub fn parity(&self) -> bool {
        self.parity
    }
}