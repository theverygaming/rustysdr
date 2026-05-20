#[derive(Debug, Clone)]
pub struct DspError {
    e: String,
}

impl DspError {
    pub fn new(s: &str) -> DspError {
        DspError {
            e: s.to_string(),
        }
    }
}

impl std::fmt::Display for DspError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.e)
    }
}
