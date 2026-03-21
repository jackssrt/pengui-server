use anyhow::Result;
pub trait ClientState {
    type IncomingPacket;
    type OutgoingPacket;
    /// dispatch a [`Self::IncomingPacket`] to an appropriate handle_ method
    async fn process_packet(&mut self, packet: Self::IncomingPacket) -> Result<()>;

    /// send a [`Self::OutgoingPacket`] to other Clients except for this one
    async fn broadcast(&mut self, packet: Self::OutgoingPacket) -> Result<()>;

    /// send a [`Self::OutgoingPacket`] to this client
    async fn send_packet(&mut self, packet: Self::OutgoingPacket) -> Result<()>;
}
