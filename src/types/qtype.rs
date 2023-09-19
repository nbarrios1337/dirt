/// The possible QTYPE field values used in resource records, as defined in [RFC 1035 3.2.2 and 3.2.3](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2.2)
///
/// We use this enum is place of all TYPE _and_ QTYPE values, for code clarity's sake.
/// > "all TYPEs are valid QTYPEs" -- RFC 1035
#[allow(clippy::upper_case_acronyms)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, num_enum::TryFromPrimitive, num_enum::IntoPrimitive,
)]
#[repr(u16)]
pub enum QType {
    /// a host address
    A = 1,
    /// an authoritative name server
    NS = 2,
    /// a mail destination (Obsolete - use MX)
    #[deprecated = "Obsoleted by RFC 973 (use MX)"]
    MD = 3,
    /// a mail forwarder (Obsolete - use MX)
    #[deprecated = "Obsoleted by RFC 973 (use MX)"]
    MF = 4,
    /// the canonical name for an alias
    CNAME = 5,
    /// marks the start of a zone of authority
    SOA = 6,
    /// a mailbox domain name (EXPERIMENTAL)
    #[deprecated = "Informally obsoleted by RFC 2505"]
    MB = 7,
    /// a mail group member (EXPERIMENTAL)
    #[deprecated = "Informally obsoleted by RFC 2505"]
    MG = 8,
    /// a mail rename domain name (EXPERIMENTAL)
    #[deprecated = "Informally obsoleted by RFC 2505"]
    MR = 9,
    /// a null RR (EXPERIMENTAL)
    NULL = 10,
    /// a well known service description
    #[deprecated = "Declared as \"not to be relied upon\" by RFC 1123 and 1127"]
    WKS = 11,
    /// a domain name pointer
    PTR = 12,
    /// host information
    HINFO = 13,
    /// mailbox or mail list information
    #[deprecated = "Informally obsoleted by RFC 2505"]
    MINFO = 14,
    /// mail exchange
    MX = 15,
    /// text strings
    TXT = 16,
    /// an IPv6 host address (see RFC 3596)
    AAAA = 28,
    // QTYPEs below
    /// A request for a transfer of an entire zone
    AXFR = 252,
    /// A request for mailbox-related records (MB, MG or MR)
    #[deprecated = "Informally obsoleted by RFC 2505"]
    MAILB = 253,
    /// A request for mail agent RRs (Obsolete - see MX)
    #[deprecated = "Obsoleted by RFC 973 (use MX)"]
    MAILA = 254,
    /// A request for all records (denoted as "*" in RFC 1035)
    ANY = 255,
}
