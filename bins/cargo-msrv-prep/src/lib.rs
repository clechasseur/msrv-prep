//! TODO

#![deny(missing_docs)]
#![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]

/// Testing
pub fn hello_world_in_bin() -> &'static str {
    "Hello, World in bin!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world_in_bin() {
        assert_eq!("Hello, World in bin!", hello_world_in_bin());
    }
}
