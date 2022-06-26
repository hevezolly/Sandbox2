use crate::field::{chunk_context::ChunkContext, neighbours::Neighbours};

use super::{movable_solids::MovableSolid, Element, liquid::Liquid, ElementType};

pub fn sand_convert(data: MovableSolid, position: (isize, isize), field: &ChunkContext) -> Element{

    if Neighbours::direct_of(position).any(|n| { field.reachable_and_fitting(n, |e| {
        if let Some(Element::Liquid(_, ElementType::Water)) = e {
            return true;
        }
        false
    })}) {
        return Element::wet_sand();
    }

    Element::MovableSolid(data, ElementType::Sand)
}