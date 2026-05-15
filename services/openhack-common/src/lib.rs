#![deny(clippy::pedantic)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    clippy::float_arithmetic,
    clippy::panic_in_result_fn,
    clippy::unimplemented,
    clippy::todo,
    clippy::dbg_macro,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::ok_expect
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::too_many_arguments,
    clippy::struct_excessive_bools,
    clippy::doc_markdown,
    clippy::unused_async,
    clippy::map_unwrap_or,
    clippy::redundant_closure_for_method_calls,
    clippy::single_match_else,
    clippy::match_same_arms,
    clippy::manual_let_else,
    clippy::needless_pass_by_value,
    clippy::unnested_or_patterns,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]

pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod health;
pub mod redis_ext;
