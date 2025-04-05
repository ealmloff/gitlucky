mod hero;
#[cfg(not(feature = "server"))]
pub use hero::Hero;
