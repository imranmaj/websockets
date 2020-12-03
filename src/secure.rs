//! Types used to customize a secure WebSocket connection; used as arguments to
//! methods on the [`WebSocketBuilder`](crate::WebSocketBuilder).

use std::fmt::{Debug, Error as FmtError, Formatter};

pub use native_tls::Protocol as TlsProtocol;
use native_tls::{Certificate, Identity};

use crate::error::WebSocketError;

// Wrapper types are necessary because the methods need to return
// Result<_, WebSocketError> not Result<_, NativeTlsError>.
// Documentation is copied from native_tls.

/// An X509 certificate.
#[derive(Clone)]
pub struct TlsCertificate(pub(crate) Certificate);

impl Debug for TlsCertificate {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str("TlsCertificate")
    }
}

impl TlsCertificate {
    /// Parses a DER-formatted X509 certificate.
    pub fn from_der(der: &[u8]) -> Result<Self, WebSocketError> {
        Ok(Self(
            Certificate::from_der(der).map_err(|e| WebSocketError::TlsConfigurationError(e))?,
        ))
    }

    /// Parses a PEM-formatted X509 certificate.
    pub fn from_pem(pem: &[u8]) -> Result<Self, WebSocketError> {
        Ok(Self(
            Certificate::from_pem(pem).map_err(|e| WebSocketError::TlsConfigurationError(e))?,
        ))
    }

    /// Returns the DER-encoded representation of this certificate.
    pub fn to_der(&self) -> Result<Vec<u8>, WebSocketError> {
        self.0
            .to_der()
            .map_err(|e| WebSocketError::TlsConfigurationError(e))
    }
}

/// A cryptographic identity.
///
/// An identity is an X509 certificate along with its corresponding private key and chain of certificates to a trusted
/// root.
#[derive(Clone)]
pub struct TlsIdentity(pub(crate) Identity);

impl Debug for TlsIdentity {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        f.write_str("TlsIdentity")
    }
}

impl TlsIdentity {
    /// Parses a DER-formatted PKCS #12 archive, using the specified password to decrypt the key.
    ///
    /// The archive should contain a leaf certificate and its private key, as well any intermediate
    /// certificates that should be sent to clients to allow them to build a chain to a trusted
    /// root. The chain certificates should be in order from the leaf certificate towards the root.
    ///
    /// PKCS #12 archives typically have the file extension `.p12` or `.pfx`, and can be created
    /// with the OpenSSL `pkcs12` tool:
    ///
    /// ```bash
    /// openssl pkcs12 -export -out identity.pfx -inkey key.pem -in cert.pem -certfile chain_certs.pem
    /// ```
    pub fn from_pkcs12(der: &[u8], password: &str) -> Result<Self, WebSocketError> {
        Ok(Self(
            Identity::from_pkcs12(der, password)
                .map_err(|e| WebSocketError::TlsConfigurationError(e))?,
        ))
    }
}
