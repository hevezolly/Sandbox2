use std::cmp;

use rand::{thread_rng, Rng};

use crate::field::chunk_context::ChunkContext;

use super::{Element, get_avalible_point, ElementData};

#[derive(Clone, Copy)]
pub struct MovableSolid{
    pub is_falling: bool, 
    pub stable_time: isize,
    pub flow_coefficient: f32,
    pub move_time: isize,
    pub keep_alive_extra_time: Option<isize>,
    pub unstuck_speed: isize,
    pub disperse_distance: isize,
    pub density: f64,
    pub slip_through_prob: f64,
}

impl PartialEq for MovableSolid {
    fn eq(&self, other: &Self) -> bool {
        self.flow_coefficient == other.flow_coefficient && self.move_time == other.move_time && self.unstuck_speed == other.unstuck_speed && self.disperse_distance == other.disperse_distance && self.density == other.density && self.slip_through_prob == other.slip_through_prob
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl MovableSolid {
    pub fn set_falling(mut self, value: bool) -> MovableSolid {self.is_falling = value; self}
    pub fn set_stable_time(mut self, value: isize) -> MovableSolid {self.stable_time = value; self}
}


impl ElementData for MovableSolid {
    type Item = MovableSolid;

    fn update(mut self, position: (isize, isize), field_access: &mut ChunkContext, convert_func: impl Fn(Self::Item, (isize, isize), &ChunkContext) -> Element ){
        let bellow_cord = (position.0, position.1 + 1);

        let move_function = |e: Element| {
            if e.solid().is_some(){
                return false;
            }
            let prob = thread_rng().gen_bool(f64::max(1. - e.density() / self.density, self.slip_through_prob));
            if let Some(data) = e.movable_solid(){
                return self != *data && prob;
            }
            prob
        };

        if field_access.reachable_empty_or_fitting(bellow_cord, move_function){
            field_access.move_from_to(position, bellow_cord, convert_func(self
                .set_stable_time(0)
                .set_falling(false), bellow_cord, field_access));
            return;
        }

        let chance = thread_rng().gen_range(
            cmp::min(0, (self.flow_coefficient * self.move_time as f32) as isize)..=
            ((self.flow_coefficient * self.move_time as f32) as isize));

        if chance >= self.stable_time {
            let side: isize = if rand::random() {1} else {-1}; 
            let shift = position.0 + side;
            let mut do_move = false;
            let mut destination = position.0 + side * self.disperse_distance;
            let adjesent = (shift, position.1);

            if field_access.reachable_empty_or_fitting(adjesent, move_function) {
                do_move = true;
            }
            else{
                let shift = position.0 - side;

                let adjesent = (shift, position.1);

                if field_access.reachable_empty_or_fitting(adjesent, move_function) {
                    do_move = true;
                    destination = position.0 - side * self.disperse_distance;
                }
            }

            if do_move {
                let destination = get_avalible_point(position, 
                    (destination, position.1), 
                    field_access, |e| {e.is_none() || move_function(e.unwrap())});
                let destination = (destination.0, destination.1 + 1);
                if field_access.reachable_empty_or_fitting(destination, |e| {
                    e.density() < self.density || move_function(e)
                }){
                    field_access.move_from_to(position, destination, 
                        convert_func(self
                            .set_stable_time(0), destination, field_access));
                }
                return;
            }
        }

        if self.stable_time < self.keep_alive_extra_time.or(Some(self.move_time)).unwrap(){
            field_access.keep_alive(position);
        }

        field_access.set_static(position, convert_func(self
            .set_stable_time(self.stable_time + 1), position, field_access));
            
    }

    fn refresh(self) -> Self::Item{
        self.set_stable_time(cmp::max(0, self.stable_time - self.unstuck_speed))
    }

    fn density(&self) -> f64 {
        self.density
    }

}