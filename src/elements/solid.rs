use super::ElementData;


#[derive(Clone, Copy)]
pub struct Solid{
    pub density: f64
}

impl ElementData for Solid{
    type Item = Solid;

    fn update(self, position: (isize, isize), field_access: &mut crate::field::chunk_context::ChunkContext, to_element: impl Fn(Self::Item, (isize, isize), &crate::field::chunk_context::ChunkContext) -> super::Element) {
        field_access.set_static(position, to_element(self, position, field_access))
    }

    fn refresh(self) -> Self::Item {
        self
    }

    fn density(&self) -> f64 {
        self.density
    }
}

