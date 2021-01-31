pub(crate) mod error;
pub(crate) mod wql;

#[derive(Debug)]
pub struct DataRegister {
    pub file_name: String,
    pub offset: usize,
    pub bytes_length: usize,
}
