use std::iter;

use rand::{thread_rng, Rng};

use crate::field::{chunk_context::ChunkContext, neighbours::Neighbours};

use super::{liquid::Liquid, Element, get_avalible_point, ElementType};

const DESOLVE_CHANCE: f64 = 0.07;

fn move_and_clear(from: (isize, isize), to: (isize, isize), data: Liquid, strength: isize, field_access: &mut ChunkContext){
    field_access.clear(from);
    let strength = strength - clear_neighbours(to, field_access, strength);
    if strength > 0{
        field_access.set(to, Element::Liquid(data, ElementType::Acid(strength)));
    }
}

fn clear_neighbours(of: (isize, isize), field_access: &mut ChunkContext, max_cleared: isize) -> isize {
    let mut removed = 0;
    for n in Neighbours::direct_of(of).chain(iter::once(of)){
        if let Ok(Some(element)) = field_access.get(n){
            match element.get_type() {
                ElementType::Acid(s) if s > 0 => (),
                ElementType::Glass => (),
                _ => {
                    if thread_rng().gen_bool(DESOLVE_CHANCE){
                        removed += 1;
                        field_access.clear(n);
                        if removed >= max_cleared{
                            break;
                        }
                    }
                }
            };
        }
    }
    removed
}


pub fn acid_update(mut data: Liquid, strength: isize, position: (isize, isize), field_access: &mut ChunkContext) {
    let bellow_cord = (position.0, position.1 + 1);

    let copy = data.clone();
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
        data.stable_time = 0;
        move_and_clear(position, bellow_cord, data, strength, field_access);
        return;
    }

    if data.stable_time < data.move_time {

        let shift = position.0 + data.side;
        let move_distance = thread_rng().gen_range(1..=data.disperse_distance);
        let mut do_move = false;
        let mut destination = position.0 + data.side * move_distance;
        let adjesent = (shift, position.1);
        

        if field_access.reachable_empty_or_fitting(adjesent, |e| e.density() < data.density) {
            do_move = true;
        }
        else{
            data.side = -data.side;
            let shift = position.0 + data.side;

            let adjesent = (shift, position.1);

            if field_access.reachable_empty_or_fitting(adjesent, |e| e.density() < data.density) {
                do_move = true;
                destination = position.0 + data.side * move_distance;
            }
        }

        if do_move {
            let destination = get_avalible_point(position, 
                (destination, position.1), 
                field_access, |element| {element.is_none() || move_func(element.unwrap())});
            let mut new_dest = (destination.0, destination.1 + 1);
            if !field_access.reachable_empty_or_fitting(new_dest, |e| e.density() < data.density){
                new_dest = destination;
            }
            if new_dest != position && field_access.reachable_empty_or_fitting(new_dest, move_func){
                data.stable_time = 0;
                move_and_clear(position, new_dest, data, strength, field_access);
                return;
            }
        }
    }
    
    if data.stable_time < data.keep_alive_extra_time.or(Some(data.move_time)).unwrap(){
        field_access.keep_alive(position);
    }
    data.stable_time += 1;
    let strength = strength - clear_neighbours(position, field_access, strength);
    if strength > 0{
        field_access.set_static(position, Element::Liquid(data, ElementType::Acid(strength)));
    }
}