static mut SEED: f32 = 13.37;
pub fn rand() -> f32 {
    unsafe {
        SEED = SEED * 9473f32 + 13f32;
        (SEED.sin() + 1.0) / 2.0
    }
}
