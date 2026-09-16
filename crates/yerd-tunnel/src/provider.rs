//! Tunnel provider definitions.
//!
//! Yerd supports Cloudflare Tunnels (`cloudflared`), ngrok, and zrok.

use std::fmt;

/// The tunneling service provider used to expose a local site publicly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TunnelProvider {
    /// Cloudflare Quick or Named tunnel (`cloudflared`).
    Cloudflare,
    /// ngrok tunnel (`ngrok`).
    Ngrok,
    /// zrok tunnel (`zrok`).
    Zrok,
}

impl TunnelProvider {
    /// Canonical CLI / configuration name of the provider.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflare",
            Self::Ngrok => "ngrok",
            Self::Zrok => "zrok",
        }
    }

    /// Expected binary executable name for this provider on the host.
    #[must_use]
    pub const fn binary_name(self) -> &'static str {
        match self {
            Self::Cloudflare => "cloudflared",
            Self::Ngrok => "ngrok",
            Self::Zrok => "zrok",
        }
    }
}

impl fmt::Display for TunnelProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_string_representations() {
        assert_eq!(TunnelProvider::Cloudflare.as_str(), "cloudflare");
        assert_eq!(TunnelProvider::Ngrok.as_str(), "ngrok");
        assert_eq!(TunnelProvider::Zrok.as_str(), "zrok");

        assert_eq!(TunnelProvider::Cloudflare.binary_name(), "cloudflared");
        assert_eq!(TunnelProvider::Ngrok.binary_name(), "ngrok");
        assert_eq!(TunnelProvider::Zrok.binary_name(), "zrok");
    }
}
