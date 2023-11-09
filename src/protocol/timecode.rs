
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct TimeCode(pub Option<u32>);

impl TimeCode {

}

impl Default for TimeCode {
    fn default() -> Self {
        TimeCode(None)
    }
}