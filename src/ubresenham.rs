use bresenham::Bresenham;

pub struct Ubresenham(Bresenham);

impl Ubresenham {
    pub fn new(first: (usize, usize), second: (usize, usize)) -> Ubresenham{
        Ubresenham(Bresenham::new((first.0 as isize, first.1 as isize), (second.0 as isize, second.1 as isize)))
    }
}

impl Iterator for Ubresenham {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.next(){
            None => None,
            Some(icord) => Some((icord.0 as usize, icord.1 as usize))
        }
    }
}