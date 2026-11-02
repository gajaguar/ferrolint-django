//! The `pylint_django` rules as a ferrolint plugin.

use ferrolint_plugin_sdk::prelude::*;

#[derive(Default)]
struct Django;

impl Checker for Django {
    fn info() -> Info {
        Info::new("django", env!("CARGO_PKG_VERSION"))
    }
}

ferrolint_plugin_sdk::export_checker!(Django);
