use rand::{thread_rng, Rng};

use crate::field::chunk_context::ChunkContext;

use super::{ElementData, Element, get_avalible_point};


#[derive(Clone, Copy)]
pub struct Liquid{
    pub side: isize,
    pub disperse_distance: isize,
    pub move_time: isize,
    pub keep_alive_extra_time: Option<isize>,
    pub density: f64,
    pub stable_time: isize,
    pub slip_through_prob: f64,
}

impl PartialEq for Liquid {
    fn eq(&self, other: &Self) -> bool {
        self.move_time == other.move_time && self.disperse_distance == other.disperse_distance && 
        self.density == other.density && self.slip_through_prob == other.slip_through_prob
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl ElementData for Liquid {
    type Item = Liquid;
    
    fn update(mut self, position: (isize, isize), field_access: &mut ChunkContext, convert_fun: impl Fn(Self::Item, (isize, isize), &ChunkContext) -> Element) {
        let bellow_cord = (position.0, position.1 + 1);

        let copy = self.clone();
        let move_func = |e: Element| {
            if e.solid().is_some(){
                return false;
            }
            let prob = thread_rng().gen_bool(f64::max(1. - e.density() / copy.density, copy.slip_through_prob));
            if let Some(data) = e.liquid(){
                return copy != *data && prob
            }
            prob
        };
        
        if field_access.reachable_empty_or_fitting(bellow_cord, move_func){
            self.stable_time = 0;
            field_access.move_from_to(position, bellow_cord, convert_fun(self, bellow_cord, field_access));
            return;
        }

        if self.stable_time < self.move_time {

            let shift = position.0 + self.side;
            let move_distance = thread_rng().gen_range(1..=self.disperse_distance);
            let mut do_move = false;
            let mut destination = position.0 + self.side * move_distance;
            let adjesent = (shift, position.1);
            

            if field_access.reachable_empty_or_fitting(adjesent, |e| e.density() < self.density) {
                do_move = true;
            }
            else{
                self.side = -self.side;
                let shift = position.0 + self.side;

                let adjesent = (shift, position.1);

                if field_access.reachable_empty_or_fitting(adjesent, |e| e.density() < self.density) {
                    do_move = true;
                    destination = position.0 + self.side * move_distance;
                }
            }

            if do_move {
                let destination = get_avalible_point(position, 
                    (destination, position.1), 
                    field_access, |element| {element.is_none() || move_func(element.unwrap())});
                let mut new_dest = (destination.0, destination.1 + 1);
                if !field_access.reachable_empty_or_fitting(new_dest, |e| e.density() < self.density){
                    new_dest = destination;
                }
                if new_dest != position && field_access.reachable_empty_or_fitting(new_dest, move_func){
                    self.stable_time = 0;
                    field_access.move_from_to(position, new_dest, convert_fun(self, new_dest, field_access));
                    return;
                }
            }
        }
        
        if self.stable_time < self.keep_alive_extra_time.or(Some(self.move_time)).unwrap(){
            field_access.keep_alive(position);
        }
        self.stable_time += 1;
        field_access.set_static(position, convert_fun(self, position, field_access));
    }

    fn refresh(mut self) -> Self::Item {
        self.stable_time = 0;
        self
    }

    fn density(&self) -> f64 {
        self.density
    }
}