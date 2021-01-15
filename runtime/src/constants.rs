pub mod currency {
    pub const SUBS: u128 = 1_000_000_000_000;
    pub const DOLLARS: u128 = SUBS; // 1_000_000_000_000
    pub const CENTS: u128 = DOLLARS / 100; // 10_000_000_000
    pub const MILLICENTS: u128 = CENTS / 1_000; // 10_000_000

    pub const fn deposit(items: u32, bytes: u32) -> u128 {
        items as u128 * 15 * CENTS + (bytes as u128) * 6 * CENTS
    }
}

pub mod time {
    pub const MILLISECS_PER_BLOCK: u64 = 6000;
    pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

    // These time units are defined in number of blocks.
    pub const MINUTES: u64 = 60_000 / (MILLISECS_PER_BLOCK as u64);
    pub const HOURS: u64 = MINUTES * 60;
    pub const DAYS: u64 = HOURS * 24;
}
