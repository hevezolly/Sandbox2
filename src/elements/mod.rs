use std::iter;

use bresenham::Bresenham;
use rand::{random, thread_rng, Rng};

use crate::field::{chunk_context::ChunkContext, neighbours::Neighbours};

use self::{movable_solids::MovableSolid, liquid::Liquid, elements_convert::{sand_convert}, solid::Solid};

pub mod movable_solids;
pub mod liquid;
mod elements_convert;
pub mod solid;

#[derive(Clone, Copy)]
pub enum ElementType{
    Sand, 
    WetSand(isize),
    Water,
    Oil,
    Block
}

#[derive(Clone, Copy)]
pub enum Element{
    MovableSolid(MovableSolid, ElementType),
    Liquid(Liquid, ElementType),
    Solid(Solid, ElementType),
}

pub trait ElementData{
    type Item;

    fn update(self, position: (isize, isize), field_access: &mut ChunkContext, to_element: impl Fn(Self::Item, (isize, isize), &ChunkContext) -> Element);
    fn refresh(self) -> Self::Item;
    fn density(&self) -> f64;
}

const WET_SAND_DRY_TIME: isize = 600;

impl Element {

    pub fn sand() -> Element{
        Element::MovableSolid(MovableSolid{ is_falling: true, 
            stable_time: 0, 
            flow_coefficient: 2.,
            move_time: 20,
            unstuck_speed: 20,
            disperse_distance: 3,
            density: 10.,
            slip_through_prob: 0.,
            keep_alive_extra_time: None,
         }, ElementType::Sand)
    }

    pub fn water() -> Element{
        Element::Liquid(Liquid{
            stable_time: 0,
            move_time: 100,
            disperse_distance: 10,
            side: if thread_rng().gen_bool(0.5) {-1} else {1},
            density: 7.,
            slip_through_prob: 0.02,
            keep_alive_extra_time: None,
        }, ElementType::Water)
    }

    pub fn wet_sand() -> Element{
        Element::MovableSolid(MovableSolid{ is_falling: true, 
            stable_time: 0, 
            flow_coefficient: 0.3,
            move_time: 10,
            unstuck_speed: 10,
            disperse_distance: 2,
            density: 10.1,
            slip_through_prob: 0.,
            keep_alive_extra_time: Some(WET_SAND_DRY_TIME),
         }, ElementType::WetSand(0))
    }

    pub fn oil() -> Element{
        Element::Liquid(Liquid{
            stable_time: 0,
            move_time: 60,
            disperse_distance: 2,
            side: if thread_rng().gen_bool(0.5) {-1} else {1},
            density: 2.,
            slip_through_prob: 0.,
            keep_alive_extra_time: None,
        }, ElementType::Oil)
    }

    pub fn block() -> Element{
        Element::Solid(Solid{
            density: 50.,
        }, ElementType::Block)
    }

    pub fn get_type(&self) -> ElementType{
        match self {
            Element::MovableSolid(_, t) => *t,
            Element::Liquid(_, t) => *t,
            Element::Solid(_, t) => *t,
        }
    }

    pub fn get_color(&self) -> [u8; 4]{
        match self.get_type() {
            ElementType::Sand => [0xff, 0xff, 0x00, 0xff],
            ElementType::Water => [0x00, 0x50, 0xff, 0xff],
            ElementType::WetSand(_) => [0xb3, 0xb3, 0x00, 0xff],
            ElementType::Oil => [0x33, 0x33, 0x10, 0xff],
            ElementType::Block => [0xb3, 0xb3, 0xb3, 0xff],
        }
    }

    pub fn update(self, position: (isize, isize), field_access: &mut ChunkContext){
        match self {
            Element::MovableSolid(data, ElementType::WetSand(t)) => data.update(position, field_access, |d, p, f| {
                let new_t = if Neighbours::direct_of(p).any(|n| {f.reachable_and_fitting(n, |e|{
                    if let Some(Element::Liquid(_, ElementType::Water)) = e {true} else {false}
                })}) {0} else {t + 1};
                if new_t >= WET_SAND_DRY_TIME {
                    Element::sand()
                }
                else {
                    Element::MovableSolid(d, ElementType::WetSand(new_t))
                }
            }),

            Element::MovableSolid(data, ElementType::Sand) => data.update(position, field_access, sand_convert),

            Element::MovableSolid(d, t) => 
                d.update(position, field_access, |d,_,_| Element::MovableSolid(d, t)), 
            Element::Solid(d, t) => 
                d.update(position, field_access, |d,_,_| Element::Solid(d, t)), 
            Element::Liquid(d, t) => 
                d.update(position, field_access, |d,_,_| Element::Liquid(d, t)), 
        }
    }

    pub fn refresh(self) -> Element{
        match self {
            Element::MovableSolid(d, t) => Element::MovableSolid(d.refresh(), t),
            Element::Liquid(d, t) => Element::Liquid(d.refresh(), t),
            Element::Solid(d, t) => Element::Solid(d.refresh(), t),
        }
    }

    pub fn density(&self) -> f64{
        match self {
            Element::MovableSolid(d, _) => d.density(),
            Element::Liquid(d, _) => d.density(),
            Element::Solid(d, _) => d.density(),
        }
    }

    pub fn solid(&self) -> Option<&Solid>{
        match self {
            Element::MovableSolid(d, _) => None,
            Element::Liquid(d, _) => None,
            Element::Solid(d, _) => Some(d),
        }
    }
    
    pub fn movable_solid(&self) -> Option<&MovableSolid>{
        match self {
            Element::MovableSolid(d, _) => Some(d),
            Element::Liquid(d, _) => None,
            Element::Solid(d, _) => None,
        }
    }

    pub fn liquid(&self) -> Option<&Liquid> {
        match self {
            Element::MovableSolid(d, _) => None,
            Element::Liquid(d, _) => Some(d),
            Element::Solid(d, _) => None,
        }
    }
}

pub fn get_avalible_point(from: (isize, isize), to: (isize, isize), chunk_access: &ChunkContext, 
fit_function: impl Fn(Option<Element>) -> bool) -> (isize, isize){
    let mut prev = from;
    for point in Bresenham::new(from, to).skip(1).chain(iter::once(to)){
        if let Ok(element) = chunk_access.get(point){
            if !fit_function(element){ 
                return prev;
            }
        }
        else{
            return prev;
        }
        prev = point;
    }
    prev
}
