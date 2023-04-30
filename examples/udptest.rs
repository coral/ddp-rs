use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("0.0.0.0:34254")?;

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        let mut buf = [0; 10];
        let (amt, src) = socket.recv_from(&mut buf)?;

        dbg!(amt, src);

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
    } // the socket is closed here
    Ok(())
}
