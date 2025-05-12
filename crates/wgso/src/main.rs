//! WGSO CLI.

use clap::Parser;
use wgso::Args;

// coverage: off (not easy to test)

fn main() {
    Args::parse().run();
}
