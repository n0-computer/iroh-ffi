use std::str::FromStr;
use std::sync::Arc;

use crate::IrohError;

/// An internet socket address, either Ipv4 or Ipv6
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketAddr {
    V4 { addr: SocketAddrV4 },
    V6 { addr: SocketAddrV6 },
}

impl From<std::net::SocketAddr> for SocketAddr {
    fn from(value: std::net::SocketAddr) -> Self {
        match value {
            std::net::SocketAddr::V4(addr) => SocketAddr::V4 {
                addr: SocketAddrV4::new(Ipv4Addr(*addr.ip()).into(), addr.port()),
            },
            std::net::SocketAddr::V6(addr) => SocketAddr::V6 {
                addr: SocketAddrV6::new(Ipv6Addr(*addr.ip()).into(), addr.port()),
            },
        }
    }
}

impl From<SocketAddr> for std::net::SocketAddr {
    fn from(value: SocketAddr) -> Self {
        match value {
            SocketAddr::V4 { addr } => {
                std::net::SocketAddr::new(std::net::IpAddr::V4(*addr.0.ip()), addr.0.port())
            }
            SocketAddr::V6 { addr } => {
                std::net::SocketAddr::new(std::net::IpAddr::V6(*addr.0.ip()), addr.0.port())
            }
        }
    }
}

impl SocketAddr {
    /// Create an Ipv4 SocketAddr
    pub fn from_ipv4(ipv4: Arc<Ipv4Addr>, port: u16) -> Self {
        SocketAddr::V4 {
            addr: SocketAddrV4::new(ipv4, port),
        }
    }

    /// Create an Ipv6 SocketAddr
    pub fn from_ipv6(ipv6: Arc<Ipv6Addr>, port: u16) -> Self {
        SocketAddr::V6 {
            addr: SocketAddrV6::new(ipv6, port),
        }
    }

    /// The type of SocketAddr
    pub fn r#type(&self) -> SocketAddrType {
        match self {
            SocketAddr::V4 { .. } => SocketAddrType::V4,
            SocketAddr::V6 { .. } => SocketAddrType::V6,
        }
    }

    /// Get the IPv4 SocketAddr representation
    pub fn as_ipv4(&self) -> Arc<SocketAddrV4> {
        match self {
            SocketAddr::V4 { addr } => Arc::new(addr.clone()),
            SocketAddr::V6 { .. } => panic!("Called SocketAddr:v4() on an Ipv6 socket addr"),
        }
    }

    /// Get the IPv6 SocketAddr representation
    pub fn as_ipv6(&self) -> Arc<SocketAddrV6> {
        match self {
            SocketAddr::V4 { .. } => panic!("Called SocketAddr:v6() on an Ipv4 socket addr"),
            SocketAddr::V6 { addr } => Arc::new(addr.clone()),
        }
    }

    /// Returns true if the two SocketAddrs have the same value
    pub fn equal(&self, other: Arc<SocketAddr>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SocketAddr::V4 { addr } => write!(f, "{}", addr),
            SocketAddr::V6 { addr } => write!(f, "{}", addr),
        }
    }
}

/// Ipv4 address
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv4Addr(pub(crate) std::net::Ipv4Addr);

impl From<std::net::Ipv4Addr> for Ipv4Addr {
    fn from(value: std::net::Ipv4Addr) -> Self {
        Ipv4Addr(value)
    }
}

impl Ipv4Addr {
    /// Create a new Ipv4 addr from 4 eight-bit octets
    pub fn new(a: u8, b: u8, c: u8, d: u8) -> Self {
        Ipv4Addr(std::net::Ipv4Addr::new(a, b, c, d))
    }

    /// Create a new Ipv4 addr from a String
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let addr = std::net::Ipv4Addr::from_str(&str).map_err(|e| IrohError::Ipv4Addr {
            description: e.to_string(),
        })?;
        Ok(Ipv4Addr(addr))
    }

    /// Get the 4 octets as bytes
    pub fn octets(&self) -> Vec<u8> {
        self.0.octets().to_vec()
    }

    /// Returns true if both Ipv4Addrs have the same value
    pub fn equal(&self, other: Arc<Ipv4Addr>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An Ipv4 socket address
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketAddrV4(pub(crate) std::net::SocketAddrV4);

impl From<std::net::SocketAddrV4> for SocketAddrV4 {
    fn from(value: std::net::SocketAddrV4) -> Self {
        SocketAddrV4(value)
    }
}

impl SocketAddrV4 {
    /// Create a new socket address from an [`Ipv4Addr`] and a port number
    pub fn new(ipv4: Arc<Ipv4Addr>, port: u16) -> Self {
        SocketAddrV4(std::net::SocketAddrV4::new(ipv4.0, port))
    }

    /// Create a new Ipv4 addr from a String
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let addr = std::net::SocketAddrV4::from_str(&str).map_err(|e| IrohError::SocketAddrV4 {
            description: e.to_string(),
        })?;
        Ok(SocketAddrV4(addr))
    }

    /// Returns the IP address associated with this socket address
    pub fn ip(&self) -> Arc<Ipv4Addr> {
        Arc::new(Ipv4Addr(*self.0.ip()))
    }

    /// Returns the port number associated with this socket address
    pub fn port(&self) -> u16 {
        self.0.port()
    }

    /// Returns true if both SocketAddrV4's have the same value
    pub fn equal(&self, other: Arc<SocketAddrV4>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for SocketAddrV4 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ipv6 address
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ipv6Addr(pub(crate) std::net::Ipv6Addr);

impl Ipv6Addr {
    #[allow(clippy::too_many_arguments)]
    pub fn new(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Self {
        Ipv6Addr(std::net::Ipv6Addr::new(a, b, c, d, e, f, g, h))
    }

    /// Create a new Ipv6 addr from a String
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let addr = std::net::Ipv6Addr::from_str(&str).map_err(|e| IrohError::Ipv6Addr {
            description: e.to_string(),
        })?;
        Ok(Ipv6Addr(addr))
    }

    /// Get the 8 sixteen-bit segments as an array
    pub fn segments(&self) -> Vec<u16> {
        self.0.segments().to_vec()
    }

    /// Returns true if both Ipv6Addr's have the same value
    pub fn equal(&self, other: Arc<Ipv6Addr>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An Ipv6 socket address
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SocketAddrV6(pub(crate) std::net::SocketAddrV6);

impl From<std::net::SocketAddrV6> for SocketAddrV6 {
    fn from(value: std::net::SocketAddrV6) -> Self {
        SocketAddrV6(value)
    }
}

impl SocketAddrV6 {
    /// Create a new socket address from an [`Ipv6Addr`] and a port number
    pub fn new(ipv6: Arc<Ipv6Addr>, port: u16) -> Self {
        SocketAddrV6(std::net::SocketAddrV6::new(ipv6.0, port, 0, 0))
    }

    /// Create a new Ipv6 addr from a String
    pub fn from_string(str: String) -> Result<Self, IrohError> {
        let addr = std::net::SocketAddrV6::from_str(&str).map_err(|e| IrohError::SocketAddrV6 {
            description: e.to_string(),
        })?;
        Ok(SocketAddrV6(addr))
    }

    /// Returns the IP address associated with this socket address
    pub fn ip(&self) -> Arc<Ipv6Addr> {
        Arc::new(Ipv6Addr(*self.0.ip()))
    }

    /// Returns the port number associated with this socket address
    pub fn port(&self) -> u16 {
        self.0.port()
    }

    /// Returns true if both SocketAddrV6's have the same value
    pub fn equal(&self, other: Arc<SocketAddrV6>) -> bool {
        *self == *other
    }
}

impl std::fmt::Display for SocketAddrV6 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of SocketAddr
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SocketAddrType {
    V4,
    V6,
}

/// A Url record
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url(pub(crate) url::Url);

impl From<url::Url> for Url {
    fn from(value: url::Url) -> Self {
        Url(value)
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Url {
    /// Get a Url from a String
    pub fn from_string(s: String) -> Result<Url, IrohError> {
        let url = url::Url::parse(&s).map_err(IrohError::url)?;
        Ok(Url(url))
    }

    /// Returns true when both Urls have the same value
    pub fn equal(&self, other: Arc<Url>) -> bool {
        *self == *other
    }
}
