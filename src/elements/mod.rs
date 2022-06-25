use std::iter;

use bresenham::Bresenham;
use rand::{random, thread_rng, Rng};

use crate::field::{chunk_context::ChunkContext, neighbours::Neighbours};

use self::{movable_solids::MovableSolid, liquid::Liquid, elements_convert::{sand_convert, water_convert}, solid::Solid};

pub mod movable_solids;
pub mod liquid;
mod elements_convert;
pub mod solid;

#[derive(Clone, Copy)]
pub enum State{
    Solid,
    Liquid
}

#[derive(Clone, Copy)]
pub enum Element{
    Sand(MovableSolid),
    WetSand(MovableSolid, isize),
    Water(Liquid),
    Oil(Liquid),
    Block(Solid),
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
        Element::Sand(MovableSolid{ is_falling: true, 
            stable_time: 0, 
            flow_coefficient: 2.,
            move_time: 20,
            unstuck_speed: 20,
            disperse_distance: 3,
            density: 10.,
            slip_through_prob: 0.,
            keep_alive_extra_time: None,
         })
    }

    pub fn water() -> Element{
        Element::Water(Liquid{
            stable_time: 0,
            move_time: 100,
            disperse_distance: 10,
            side: if thread_rng().gen_bool(0.5) {-1} else {1},
            density: 7.,
            slip_through_prob: 0.02,
            keep_alive_extra_time: None,
        })
    }

    pub fn wet_sand() -> Element{
        Element::WetSand(MovableSolid{ is_falling: true, 
            stable_time: 0, 
            flow_coefficient: 0.3,
            move_time: 10,
            unstuck_speed: 10,
            disperse_distance: 2,
            density: 10.1,
            slip_through_prob: 0.,
            keep_alive_extra_time: Some(WET_SAND_DRY_TIME),
         }, 0)
    }

    pub fn oil() -> Element{
        Element::Oil(Liquid{
            stable_time: 0,
            move_time: 60,
            disperse_distance: 2,
            side: if thread_rng().gen_bool(0.5) {-1} else {1},
            density: 2.,
            slip_through_prob: 0.,
            keep_alive_extra_time: None,
        })
    }

    pub fn block() -> Element{
        Element::Block(Solid{
            density: 50.,
        })
    }

    pub fn get_color(&self) -> [u8; 4]{
        match self {
            Element::Sand(d) => [0xff, 0xff, 0x00, 0xff],
            Element::Water(d) => [0x00, 0x50, 0xff, 0xff],
            Element::WetSand(_, _) => [0xb3, 0xb3, 0x00, 0xff],
            Element::Oil(_) => [0x33, 0x33, 0x10, 0xff],
            Element::Block(_) => [0xb3, 0xb3, 0xb3, 0xff],
        }
    }

    pub fn update(self, position: (isize, isize), field_access: &mut ChunkContext){
        match self {
            Element::Sand(data) => data.update(position, field_access, sand_convert),
            Element::Water(data) => data.update(position, field_access, water_convert),
            Element::WetSand(data, t) => data.update(position, field_access, |d, p, f| {
                let new_t = if Neighbours::direct_of(p).any(|n| {f.reachable_and_fitting(n, |e|{
                    if let Some(Element::Water(_)) = e {true} else {false}
                })}) {0} else {t + 1};
                if new_t >= WET_SAND_DRY_TIME {
                    Element::sand()
                }
                else {
                    Element::WetSand(d, new_t)
                }
            }),
            Element::Oil(data) => data.update(position, field_access, |d, _, _| Element::Oil(d)),
            Element::Block(d) => d.update(position, field_access, |d, _, _| Element::Block(d)),
        }
    }

    pub fn refresh(self) -> Element{
        match self {
            Element::Sand(data) => Element::Sand(data.refresh()),
            Element::Water(data) => Element::Water(data.refresh()),
            Element::WetSand(data,t) => Element::WetSand(data.refresh(), t),
            Element::Oil(d) => Element::Oil(d.refresh()),
            Element::Block(d) => Element::Block(d),
        }
    }

    pub fn density(&self) -> f64{
        match self {
            Element::Sand(d) => d.density(),
            Element::WetSand(d,_) => d.density(),
            Element::Water(d) => d.density(),
            Element::Oil(d) => d.density(),
            Element::Block(d) => d.density(),
        }
    }

    pub fn solid(&self) -> Option<&Solid>{
        match self {
            Element::Sand(_) => None,
            Element::WetSand(_,_) => None,
            Element::Water(_) => None,
            Element::Oil(_) => None,
            Element::Block(d) => Some(d),
        }
    }
    
    pub fn movable_solid(&self) -> Option<&MovableSolid>{
        match self {
            Element::Sand(d) => Some(d),
            Element::WetSand(d,_) => Some(d),
            Element::Water(_) => None,
            Element::Oil(_) => None,
            Element::Block(_) => None,
        }
    }

    pub fn liquid(&self) -> Option<&Liquid> {
        match self {
            Element::Sand(_) => None,
            Element::WetSand(_,_) => None,
            Element::Water(d) => Some(d),
            Element::Oil(d) => Some(d),
            Element::Block(_) => None,
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
