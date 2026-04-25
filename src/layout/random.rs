use crate::LayoutOptions;

use super::LayoutNode;

pub(crate) fn layout(nodes: &mut [LayoutNode], options: &LayoutOptions) {
    let mut rng = SplitMix64::new(options.seed.unwrap_or(1));
    for node in nodes {
        node.x = options.center_x + rng.next_unit();
        node.y = options.center_y + rng.next_unit();
    }
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_unit(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d049bb133111eb);
        value ^= value >> 31;
        (value as f64) / (u64::MAX as f64)
    }
}
