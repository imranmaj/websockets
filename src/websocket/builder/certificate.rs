use native_tls::Certificate as NativeTlsCertificate;

use crate::error::WebSocketError;

pub struct Certificate(NativeTlsCertificate);

impl Certificate {
    pub fn from_der(der: &[u8]) -> Result<Self, WebSocketError> {
        Ok(Self(
            NativeTlsCertificate::from_der(der).map_err(|e| WebSocketError::CertificateError(e))?,
        ))
    }

    pub fn from_pem(pem: &[u8]) -> Result<Self, WebSocketError> {
        Ok(Self(
            NativeTlsCertificate::from_pem(pem).map_err(|e| WebSocketError::CertificateError(e))?,
        ))
    }

    pub fn to_der(&self) -> Result<Vec<u8>, WebSocketError> {
        self.0
            .to_der()
            .map_err(|e| WebSocketError::DerConversionError(e))
    }
}

impl Into<NativeTlsCertificate> for Certificate {
    fn into(self) -> NativeTlsCertificate {
        self.0
    }
}
