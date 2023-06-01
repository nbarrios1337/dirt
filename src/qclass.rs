/// The possible QCLASS field values used in resource records, as defined in [RFC 1035 3.2.4 and 3.2.5](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.4)
///
/// We use this enum is place of all CLASS _and_ QCLASS values, for code clarity's sake.
/// > "every CLASS is a valid QCLASS" -- RFC 1035
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, num_enum::TryFromPrimitive, num_enum::IntoPrimitive,
)]
#[repr(u16)]
pub enum QClass {
    /// the Internet
    IN = 1,
    /// the CSNET class (Obsolete - used only for examples in some obsolete RFCs)
    CS = 2,
    /// the CHAOS class
    CH = 3,
    /// Hesiod [Dyer 87]
    HS = 4,
    /// any class (denoted as "*" in RFC 1035)
    ALL = 255,
}
