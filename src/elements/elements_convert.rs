use crate::field::{chunk_context::ChunkContext, neighbours::Neighbours};

use super::{movable_solids::MovableSolid, Element, liquid::Liquid};

pub fn sand_convert(data: MovableSolid, position: (isize, isize), field: &ChunkContext) -> Element{

    if Neighbours::direct_of(position).any(|n| { field.reachable_and_fitting(n, |e| {
        if let Some(Element::Water(_)) = e {
            return true;
        }
        false
    })}) {
        return Element::wet_sand();
    }

    Element::Sand(data)
}

pub fn water_convert(data: Liquid, position: (isize, isize), field: &ChunkContext) -> Element{
    Element::Water(data)
}