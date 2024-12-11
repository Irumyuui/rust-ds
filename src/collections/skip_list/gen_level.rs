use crate::collections::skip_list::MAX_LEVEL;

pub trait LevelGenerator {
    /// requires: 0 < result <= 32
    fn random_level(&mut self) -> usize;
}

#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultGenerator;

impl LevelGenerator for DefaultGenerator {
    fn random_level(&mut self) -> usize {
        const P: f64 = 0.6;
        let mut level = 1;
        let mut x = P;

        let f = 1. - rand::random::<f64>();
        while x > f && level < MAX_LEVEL {
            level += 1;
            x *= P;
        }

        level
    }
}

#[cfg(test)]
mod tests {
    use crate::collections::skip_list::{LevelGenerator, MAX_LEVEL};

    use super::DefaultGenerator;

    #[test]
    fn default_generator() {
        let mut gen = DefaultGenerator::default();

        for _ in 0..1000000 {
            let level = gen.random_level();
            assert!(0 < level && level <= MAX_LEVEL);
        }
    }
}
