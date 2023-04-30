use std::net::UdpSocket;

fn main() -> std::io::Result<()> {
    {
        let socket = UdpSocket::bind("0.0.0.0:4048")?;
        //socket.connect("10.0.1.9:4048").unwrap();

        // Receives a single datagram message on the socket. If `buf` is too small to hold
        // the message, it will be cut off.
        loop {
            let mut buf = [0; 1500];
            let (amt, src) = socket.recv_from(&mut buf)?;

            dbg!(src, amt);
        }

        // Redeclare `buf` as slice of the received data and send reverse data back to origin.
    } // the socket is closed here
    Ok(())
}
