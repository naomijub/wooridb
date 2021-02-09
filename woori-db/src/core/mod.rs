use ron::ser::PrettyConfig;

pub(crate) mod wql;

pub fn pretty_config_output() -> PrettyConfig {
    PrettyConfig::new()
        .with_separate_tuple_members(true)
        .with_decimal_floats(true)
        .with_indentor(" ".to_string())
        .with_new_line("\n".to_string())
}

pub fn pretty_config_inner() -> PrettyConfig {
    PrettyConfig::new()
        .with_indentor("".to_string())
        .with_new_line("".to_string())
}
