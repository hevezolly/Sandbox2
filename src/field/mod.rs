pub mod rect;
pub mod chunk;
pub mod chunk_context;
pub mod neighbours;
pub mod threads;

use std::{collections::{HashMap, HashSet}, rc::Rc, borrow::{BorrowMut, Borrow}, ops::{DerefMut, Deref}, cell::RefCell, sync::{Arc, Mutex, mpsc::{Sender, Receiver, self}, RwLock}, thread::JoinHandle};

use crate::elements::Element;

use self::{chunk::{Chunk, CHUNK_SIZE, CordInChunk}, chunk_context::{ChunkContext, UnsolvedActions::{MissingChunkInsertion, self}}, neighbours::Neighbours, rect::Rect, threads::{TaskMessage, ResultMessage, spawn_field_worker_thread}};


const CHUNK_ISIZE: (isize, isize) = (CHUNK_SIZE.0 as isize, CHUNK_SIZE.1 as isize);


type ChunkRef = Arc<RwLock<Chunk>>;

pub struct Field{
    chunks: HashMap<(isize, isize), ChunkRef>,
    thread_handles: Vec<JoinHandle<()>>,
    task_sender: Sender<TaskMessage>,
    result_receiver: Receiver<ResultMessage>,
    chunks_update_order: Vec<HashSet<(isize, isize)>>,
    chunk_boundaries: Rect<isize>,
    updated_cells: Vec<(isize, isize)>,
    parity: bool
}

pub type ChunkCord = (isize, isize);

pub fn global_cord_to_chunk_local(position: (isize, isize)) -> (ChunkCord, CordInChunk){
    let chunk_cord = (position.0 / CHUNK_ISIZE.0, position.1 / CHUNK_ISIZE.1);
    let cord_in_chunk = (position.0.rem_euclid(CHUNK_ISIZE.0) as usize, position.1.rem_euclid(CHUNK_ISIZE.1) as usize);
    (chunk_cord, cord_in_chunk)
}

pub fn local_cord_to_global(cord_in_chunk: CordInChunk, chunk_cord: ChunkCord) -> (isize, isize){
    (chunk_cord.0 * CHUNK_ISIZE.0 + cord_in_chunk.0 as isize, chunk_cord.1 * CHUNK_ISIZE.1 + cord_in_chunk.1 as isize)
}

impl Field {

    pub fn new(max_field_chunk_count: (usize, usize), number_of_threads: usize) -> Field{

        let (task_sender, task_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();

        let task_receiver = Arc::new(Mutex::new(task_receiver));

        let handles:Vec<JoinHandle<()>> = (0..number_of_threads).map(|i| {
            spawn_field_worker_thread(task_receiver.clone(), result_sender.clone(), i)
        }).collect();

        Field { chunks: HashMap::new(), 
            chunk_boundaries: Rect::from((0, 0), (max_field_chunk_count.0 as isize, max_field_chunk_count.1 as isize)),
            parity: false,
            thread_handles: handles,
            task_sender,
            result_receiver,
            chunks_update_order: vec![HashSet::new();4],
            updated_cells: Vec::new(), }
    }

    pub fn get(&self, position: (isize, isize)) -> Option<Element>{
        let (chunk_c, c_in_chunk) = global_cord_to_chunk_local(position);
        return self.chunks.get(&chunk_c)?.read().unwrap().get(c_in_chunk);
    }

    pub fn get_chunks(&self) -> Vec<Rect<isize>> {
        let mut rects = Vec::new();

        for (cord, _) in self.chunks.iter(){
            let top_left = (cord.0 * CHUNK_ISIZE.0, cord.1 * CHUNK_ISIZE.1);
            let bottom_right = ((cord.0 + 1) * CHUNK_ISIZE.0, (cord.1 + 1) * CHUNK_ISIZE.1);
            rects.push(Rect::from(top_left, bottom_right));
        }

        rects
    }

    pub fn get_chunks_update_rects(&self) -> Vec<Rect<isize>>{
        let mut rects = Vec::new();

        for (cord, chunk) in self.chunks.iter(){
            let rect = chunk.read().unwrap().get_update_rect();
            let top_left = (cord.0 * CHUNK_ISIZE.0 + rect.left() as isize, 
            cord.1 * CHUNK_ISIZE.1 + rect.top() as isize);
            let bottom_right = (cord.0  * CHUNK_ISIZE.0 + rect.right() as isize, 
            cord.1 * CHUNK_ISIZE.1 + rect.bottom() as isize);
            rects.push(Rect::from(top_left, bottom_right));
        }

        rects
    }

    pub fn load_pixels(&mut self) -> Vec<((isize, isize), [u8;4])>{
        let mut result = Vec::new();
        for updated_pix in self.updated_cells.iter(){
            let (chunk_cord, cord_in_chunk) = global_cord_to_chunk_local(*updated_pix);
            let chunk = self.chunks.get(&chunk_cord);
            let color = if let Some(chunk) = chunk{
                match chunk.read().unwrap().get(cord_in_chunk) {
                    Some(element) => element.get_color(),
                    None => [0x00, 0x00, 0x00, 0xff],
                }
            }
            else{
                [0x00, 0x00, 0x00, 0xff]
            };
            result.push((*updated_pix, color));
        }
        self.updated_cells.clear();
        result
    }

    pub fn set_in_area(&mut self, position: (isize, isize), size: (usize, usize), element: Option<Element>){
        let mut top = position.1 - size.1 as isize/2;
        if top < 0{
            top = 0;
        }
        let mut left = position.0 - size.0 as isize/2;
        if left < 0{
            left = 0;
        }
        let mut bottom = position.1+size.1 as isize/2+size.1 as isize%2;
        if bottom >= self.chunk_boundaries.bottom() * CHUNK_ISIZE.1 {
            bottom = self.chunk_boundaries.bottom() * CHUNK_ISIZE.1;
        }
        let mut right = position.0+size.0 as isize/2+size.0 as isize%2;
        if right >= self.chunk_boundaries.right() * CHUNK_ISIZE.0 {
            right = self.chunk_boundaries.right() * CHUNK_ISIZE.0;
        }
        for y in top..bottom{
            for x in left..right{
                self.set((x, y), element);
            }
        }
    }

    pub fn set(&mut self, position: (isize, isize), element: Option<Element>){
        let (chunk_c, c_in_chunk) = global_cord_to_chunk_local(position);
        if !self.chunks.contains_key(&chunk_c){
            self.insert_chunk(chunk_c);
        }
        let chunk = self.chunks.get(&chunk_c).unwrap();
        match element {
            Some(e) => chunk.write().unwrap().set(c_in_chunk, e, self.parity),
            None => chunk.write().unwrap().clear(c_in_chunk),
        }
        chunk.write().unwrap().add_point_in_update_cycle_with_neighbourhood(c_in_chunk);
        self.updated_cells.push(position);
    }

    fn remove_empty_chunks(&mut self){
        let mut empty_chunks = Vec::new();
        for (chunk_cord, chunk) in self.chunks.iter(){
            if chunk.read().unwrap().number_of_elements() == 0 {
                empty_chunks.push(*chunk_cord);
            }
        }
        for empty_chunk_cord in empty_chunks{
            self.delete_chunk(empty_chunk_cord);
        }
    }

    fn get_chunk_order(cord: ChunkCord) -> usize{
        (cord.0.rem_euclid(2) + (cord.1.rem_euclid(2) * 2)) as usize
    }

    fn insert_chunk(&mut self, cord: ChunkCord){
        self.chunks.insert(cord,  Arc::new(RwLock::new(Chunk::new(self.parity))));
        // let order = Field::get_chunk_order(cord);
        // self.chunks_update_order[order].insert(cord);
    }

    fn delete_chunk(&mut self, cord: ChunkCord){
        self.chunks.remove(&cord);
        // let order = Field::get_chunk_order(cord);
        // self.chunks_update_order[order].remove(&cord);
    }

    fn solve_unsolved_action(&mut self, unsolved_actions: Vec<UnsolvedActions>){
        for unsolved_action in unsolved_actions{
            match unsolved_action {
                MissingChunkInsertion { chunk_cord, insertion_cord, elementToInsert } =>{
                    if !self.chunks.contains_key(&chunk_cord){
                        self.insert_chunk(chunk_cord);
                    }
                    self.chunks.get(&chunk_cord).unwrap().write().unwrap().set(insertion_cord, elementToInsert, !self.parity);
                },
            }
        }
    }

    pub fn update(&mut self){

        for (_, c) in self.chunks.iter(){
            c.write().unwrap().update_rect();
        }

        let mut unsolved_actions = Vec::new();

        // for update_order in 0..4 {
            let mut spawned = 0;
            for (chunk_cord, chunk) in self.chunks.iter(){

                if !self.chunks.get(chunk_cord).unwrap().read().unwrap().needs_updates(){
                    continue;
                }

                let mut neighbours = HashMap::new();
                for neighbour in Neighbours::of(*chunk_cord).with_boundaries(self.chunk_boundaries){
                    let chunk_opt = self.chunks.get(&neighbour);
                    if let Some(chunk) = chunk_opt{
                        neighbours.insert(neighbour, Some(chunk.clone()));
                    }
                    else{
                        neighbours.insert(neighbour, None);
                    }
                }

                let chunk = chunk.clone();

                self.task_sender.send(TaskMessage::UpdateChunk(ChunkContext::new(
                    chunk, 
                    *chunk_cord, 
                    neighbours, 
                    self.parity))).ok();
                
                spawned += 1;
            }

            for _ in 0..spawned{
                match self.result_receiver.recv().unwrap() {
                    ResultMessage::UnsolvedActions { unsolved, updated } => {
                        unsolved_actions.extend(unsolved);
                        self.updated_cells.extend(updated);
                    },
                }
            }

        // }
        self.parity = !self.parity;

        self.solve_unsolved_action(unsolved_actions);

        self.remove_empty_chunks();

        
    }
}