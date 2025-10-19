//! Error types for DDP operations.
//!
//! This module defines all error types that can occur when working with DDP connections.

use thiserror::Error;

/// Errors that can occur during DDP operations.
///
/// All errors implement the standard [`std::error::Error`] trait via `thiserror`.
#[derive(Error, Debug)]
pub enum DDPError {
    /// Socket or network I/O error
    #[error("socket error")]
    Disconnect(#[from] std::io::Error),

    /// Failed to resolve the provided address
    #[error("No valid socket addr found")]
    NoValidSocketAddr,

    /// JSON parsing error for control messages
    #[error("parse error")]
    ParseError(#[from] serde_json::Error),

    /// Received data from an unknown or unexpected client
    #[error("invalid sender, did you forget to connect() ( data from {from:?} - {data:?})")]
    UnknownClient {
        /// The address that sent the unexpected data
        from: std::net::SocketAddr,
        /// The unexpected data received
        data: Vec<u8>,
    },

    /// Received packet with invalid format or structure
    #[error("Invalid packet")]
    InvalidPacket,

    /// No packets are currently available to receive (non-blocking operation)
    #[error("There are no packets waiting to be read. This error should be handled explicitly")]
    NothingToReceive,

    /// Error from the internal packet receiver channel
    #[error("Error receiving packet: {0}")]
    CrossBeamError(#[from] crossbeam::channel::TryRecvError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn test_error_display_disconnect() {
        let io_error = std::io::Error::new(std::io::ErrorKind::ConnectionReset, "connection reset");
        let error = DDPError::Disconnect(io_error);
        assert_eq!(error.to_string(), "socket error");
    }

    #[test]
    fn test_error_display_no_valid_socket_addr() {
        let error = DDPError::NoValidSocketAddr;
        assert_eq!(error.to_string(), "No valid socket addr found");
    }

    #[test]
    fn test_error_display_parse_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("{invalid json")
            .expect_err("should fail to parse");
        let error = DDPError::ParseError(json_error);
        assert_eq!(error.to_string(), "parse error");
    }

    #[test]
    fn test_error_display_unknown_client() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 8080);
        let data = vec![0x01, 0x02, 0x03];
        let error = DDPError::UnknownClient {
            from: addr,
            data: data.clone(),
        };

        let error_str = error.to_string();
        assert!(error_str.contains("invalid sender"));
        assert!(error_str.contains("192.168.1.1:8080"));
        assert!(error_str.contains("[1, 2, 3]"));
    }

    #[test]
    fn test_error_display_invalid_packet() {
        let error = DDPError::InvalidPacket;
        assert_eq!(error.to_string(), "Invalid packet");
    }

    #[test]
    fn test_error_display_nothing_to_receive() {
        let error = DDPError::NothingToReceive;
        assert_eq!(
            error.to_string(),
            "There are no packets waiting to be read. This error should be handled explicitly"
        );
    }

    #[test]
    fn test_error_display_crossbeam_error() {
        use crossbeam::channel::TryRecvError;

        let error = DDPError::CrossBeamError(TryRecvError::Empty);
        assert!(error.to_string().contains("Error receiving packet"));
    }

    #[test]
    fn test_error_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "broken pipe");
        let error: DDPError = io_error.into();

        match error {
            DDPError::Disconnect(_) => {},
            _ => panic!("Expected Disconnect variant"),
        }
    }

    #[test]
    fn test_error_from_json_error() {
        let json_error = serde_json::from_str::<serde_json::Value>("{bad}")
            .expect_err("should fail");
        let error: DDPError = json_error.into();

        match error {
            DDPError::ParseError(_) => {},
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_error_from_crossbeam_error() {
        use crossbeam::channel::TryRecvError;

        let crossbeam_error = TryRecvError::Disconnected;
        let error: DDPError = crossbeam_error.into();

        match error {
            DDPError::CrossBeamError(_) => {},
            _ => panic!("Expected CrossBeamError variant"),
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = DDPError::InvalidPacket;
        let debug_str = format!("{:?}", error);
        assert_eq!(debug_str, "InvalidPacket");
    }

    #[test]
    fn test_unknown_client_error_fields() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 4048);
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];

        let error = DDPError::UnknownClient {
            from: addr.clone(),
            data: data.clone(),
        };

        match error {
            DDPError::UnknownClient { from, data: d } => {
                assert_eq!(from, addr);
                assert_eq!(d, data);
            }
            _ => panic!("Expected UnknownClient variant"),
        }
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DDPError>();
    }
}
