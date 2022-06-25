use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender}, RwLock}, thread, collections::HashMap};

use crate::elements::ElementData;

use super::{chunk::Chunk, chunk_context::{ChunkContext, UnsolvedActions}, local_cord_to_global, ChunkCord, ChunkRef};



pub enum TaskMessage {
    UpdateChunk(ChunkContext)
}

pub enum ResultMessage {
    UnsolvedActions{
    unsolved: Vec<UnsolvedActions>, 
    updated: Vec<(isize, isize)>
    }
}

pub fn spawn_field_worker_thread(task_reciver: Arc<Mutex<Receiver<TaskMessage>>>, result_sender: Sender<ResultMessage>, index: usize) -> thread::JoinHandle<()>{
    thread::spawn(move ||{
        let reciver = task_reciver;
        let sender = result_sender;
        loop {
            let task = {reciver.lock().unwrap().recv().unwrap()};
            match task{
                TaskMessage::UpdateChunk(context) => {
                    let (unsolved, updated) = update_chunk(context);
                    sender.send(ResultMessage::UnsolvedActions{
                        unsolved,
                        updated,
                    });
                },
            }
        }
    })
}

fn update_chunk(mut chunk_context: ChunkContext) -> (Vec<UnsolvedActions>, Vec<(isize, isize)>){
    let update_rect = chunk_context.current_chunk().read().unwrap().get_update_rect();
    let height = update_rect.bottom() - update_rect.top();
    let width = update_rect.right() - update_rect.left();
    let p = if chunk_context.parity() {1} else {0};
    for y in ((0..(height/2+(height * p)%2)).rev().map(|y| 1 - p + y*2)
                    .chain((0..height/2+(height * (1-p))%2).map(|y| y*2 + p)))
                    .map(|y| {y + update_rect.top()}).rev(){
        for x in ((0..(width/2+(width * p)%2)).rev().map(|x| 1 - p + x*2)
                    .chain((0..width/2+(width * (1-p))%2).map(|x| x*2 + p)))
                    .map(|x| {x + update_rect.left()}){
            let local_cord = (x, y);
            let (element, should_update) =  {
                let parity = chunk_context.current_chunk().read().unwrap().parity(local_cord);
                let element = chunk_context.current_chunk().read().unwrap().get(local_cord);
                (element, parity == chunk_context.parity())
            };
            if let Some(element) = element{
                if should_update{
                    element.update(local_cord_to_global(local_cord,chunk_context.current_chunk_cord()), 
                    &mut chunk_context);
                }
                else{
                    chunk_context.keep_alive_local(local_cord);
                }
            }
        }
    }
    (chunk_context.unsolved_actions, chunk_context.updated_coordinates)
}