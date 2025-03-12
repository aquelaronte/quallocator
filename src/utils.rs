/*
 * Align passed size in 8 bytes multiplier
 */
pub fn align_up(size: i32) -> i32 {
    (size + (8 - 1)) & !(8 - 1)
}
