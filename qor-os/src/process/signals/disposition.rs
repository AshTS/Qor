/// Signal Dispositions (default behavior if no handler has been created)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalDisposition
{
    Terminate,
    Ignore,
    Core,
    Stop,
    Continue
}