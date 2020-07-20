use native_tls::Protocol as NativeTlsProtocol;

#[derive(Debug, Copy, Clone)]
pub enum Protocol {
    /// The SSL 3.0 protocol.
    ///
    /// # Warning
    ///
    /// SSL 3.0 has severe security flaws, and should not be used unless absolutely necessary. If
    /// you are not sure if you need to enable this protocol, you should not.
    Sslv3,
    /// The TLS 1.0 protocol.
    Tlsv10,
    /// The TLS 1.1 protocol.
    Tlsv11,
    /// The TLS 1.2 protocol.
    Tlsv12,
}

impl Into<NativeTlsProtocol> for Protocol {
    fn into(self) -> NativeTlsProtocol {
        match self {
            Self::Sslv3 => NativeTlsProtocol::Sslv3,
            Self::Tlsv10 => NativeTlsProtocol::Tlsv10,
            Self::Tlsv11 => NativeTlsProtocol::Tlsv11,
            Self::Tlsv12 => NativeTlsProtocol::Tlsv12,
        }
    }
}
