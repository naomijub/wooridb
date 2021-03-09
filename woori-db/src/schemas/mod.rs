use ron::ser::PrettyConfig;

pub mod error;
pub mod history;
pub mod query;
pub mod tx;

pub fn pretty_config() -> PrettyConfig {
    PrettyConfig::new()
        .with_separate_tuple_members(true)
        .with_decimal_floats(true)
        .with_indentor(" ".to_string())
        .with_new_line("\n".to_string())
}
