use native_tls::Identity as NativeTlsIdentity;

use crate::error::WebSocketError;

pub struct Identity(NativeTlsIdentity);

impl Identity {
    pub fn from_pkcs12(der: &[u8], password: &str) -> Result<Self, WebSocketError> {
        Ok(Self(
            NativeTlsIdentity::from_pkcs12(der, password)
                .map_err(|e| WebSocketError::IdentityParseError(e))?,
        ))
    }
}

impl Into<NativeTlsIdentity> for Identity {
    fn into(self) -> NativeTlsIdentity {
        self.0
    }
}
