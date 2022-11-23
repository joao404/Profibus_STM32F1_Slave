use heapless::{
    Vec,
};
enum DpSlaveState
{
    Por = 1,  // Power on reset
    Wrpm = 2, // Wait for parameter
    Wcfg = 3, // Wait for config
    Ddxchg = 4// Data exchange
}

pub struct PbDpSlave{
    buffer: Vec<u8, 255>,
    state : DpSlaveState,
}

impl PbDpSlave{
    pub fn new() ->Self{
        Self{
            buffer : Vec::<u8, 255>::new(),
            state : DpSlaveState::Por,
        }
    }

    pub fn handle_data(&mut self, data : u8) {
        match self.state{
            DpSlaveState::Ddxchg=>self.buffer.push(data).unwrap(),
            _=>self.buffer.push(data).unwrap(),
        }
    }

    pub fn handle_message_timeout(&mut self)
    {
        let _test = self.buffer.len();

        self.buffer.clear();
    }
}