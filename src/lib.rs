#![feature(decl_macro)]

pub mod bound;
pub mod interval;
pub mod interval_map;

pub use self::interval::Interval;
pub use self::interval_map::IntervalMap;
