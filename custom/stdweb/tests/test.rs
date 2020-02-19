// Explicitly use the Custom RNG crate to link it in.
use stdweb_getrandom as _;

use getrandom::getrandom;
use test;
#[path = "../../../src/test_common.rs"]
mod test_common;
