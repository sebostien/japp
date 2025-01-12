pub struct Interleaved<A, B> {
    first: bool,
    take_a: bool,
    iter_a: A,
    iter_b: B,
}

impl<A, B> Interleaved<A, B> {
    pub fn new(iter_a: A, iter_b: B) -> Self {
        Self {
            first: true,
            take_a: true,
            iter_a,
            iter_b,
        }
    }
}

impl<A, B, T> Iterator for Interleaved<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first {
            self.first = false;
            self.iter_a.next()
        } else if self.take_a {
            self.take_a = false;
            self.iter_a.next()
        } else {
            self.take_a = true;
            self.iter_b.next()
        }
    }
}

pub trait Interleave<A, B> {
    fn interleave(self, other: B) -> Interleaved<A, B>;
}

impl<A, B, T> Interleave<A, B> for A
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    /// Interleaves `self` and `other`.
    ///  
    /// First produces two items from `self`, then `1` out of other.
    /// After this they are mixed.
    fn interleave(self, other: B) -> Interleaved<A, B> {
        Interleaved::new(self, other)
    }
}

#[cfg(test)]
mod tests {
    use super::Interleave;

    #[test]
    fn interleave() {
        let inter = (1..=5).interleave(10..).collect::<Vec<_>>();

        assert_eq!(inter, vec![1, 2, 10, 3, 11, 4, 12, 5, 13])
    }
}
