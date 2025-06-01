#[derive(Debug)]
pub struct BotConfig {
    pub k_p: f32,
    pub k_d: f32,
}

impl BotConfig {
    pub const fn default() -> Self {
        Self {
            k_p: 0.05,
            k_d: 0.001,
        }
    }
}
