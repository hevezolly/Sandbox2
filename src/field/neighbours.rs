use super::rect::Rect;


#[derive(Clone)]
pub struct Neighbours{
    current: usize,
    all_neighbours: Vec<(isize, isize)>,
    boundaries: Option<Rect<isize>>
}

impl Neighbours {
    pub fn of(position: (isize, isize)) -> Neighbours{
        Neighbours { current: 0, all_neighbours: vec![
            (position.0 - 1, position.1 - 1),
            (position.0    , position.1 - 1),
            (position.0 + 1, position.1 - 1),
            (position.0 - 1, position.1    ),
            (position.0 + 1, position.1    ),
            (position.0 - 1, position.1 + 1),
            (position.0    , position.1 + 1),
            (position.0 + 1, position.1 + 1),
        ],
            boundaries: None, }
    }

    pub fn direct_of(position: (isize, isize)) -> Neighbours{
        Neighbours { current: 0, all_neighbours: vec![
            (position.0    , position.1 - 1),
            (position.0 - 1, position.1    ),
            (position.0 + 1, position.1    ),
            (position.0    , position.1 + 1),
        ],
        boundaries: None, }
    }

    pub fn horisontal_of(position: (isize, isize)) -> Neighbours{
        Neighbours { current: 0, all_neighbours: vec![
            (position.0 - 1, position.1    ),
            (position.0 + 1, position.1    ),
        ],
        boundaries: None, }
    }
    
    pub fn vertical_of(position: (isize, isize)) -> Neighbours{
        Neighbours { current: 0, all_neighbours: vec![
            (position.0    , position.1 - 1),
            (position.0    , position.1 + 1),
        ],
        boundaries: None, }
    }

    pub fn with_boundaries(mut self, boundaries: Rect<isize>) -> Neighbours{
        self.boundaries = Some(boundaries);
        self
    }
}

impl Iterator for Neighbours {
    type Item = (isize, isize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.all_neighbours.len(){
            return None;
        }
        let result = self.all_neighbours[self.current];
        self.current += 1;
        if let Some(boundary) = self.boundaries{
            if !boundary.is_inside(result){
                return self.next();
            }
        }
        Some(result)
    }
}