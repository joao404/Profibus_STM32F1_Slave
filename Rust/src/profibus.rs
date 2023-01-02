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

enum StreamState
{
    WaitSyn,
    WaitData,
    GetData,
    WaitMinTsdr,
    SendData
}

pub struct PbDpSlave{
    buffer: Vec<u8, 255>,
    slave_state : DpSlaveState,
    stream_state : StreamState,
}

impl PbDpSlave{
    pub fn new() ->Self{
        Self{
            buffer : Vec::<u8, 255>::new(),
            slave_state : DpSlaveState::Por,
            stream_state : StreamState::WaitSyn,
        }
    }

    pub fn handle_rx(&mut self, data : u8) {
        self.buffer.push(data).unwrap();

  // if we waited for TSYN, data can be saved
  if (StreamState::WaitData == self.stream_state)
  {
    self.stream_state = StreamState::GetData;
  }

  // Einlesen erlaubt?
  if (StreamState::GetData == self.stream_state)
  {
    m_rxBufCnt++;

    // Nicht mehr einlesen als in Buffer reinpasst
    if (m_rxBufCnt >= m_config.bufSize)
      m_rxBufCnt--;
  }

  // Profibus Timer ruecksetzen
  setTimerCounter(0);
  clearOverflowFlag();

    }

    pub fn handle_message_timeout(&mut self)
    {
        let _test = self.buffer.len();

        self.buffer.clear();
    }
}

pub struct PbDpSlaveStm32f1{
    buffer: Vec<u8, 255>,
    slave_state : DpSlaveState,
    stream_state : StreamState,
}

impl PbDpSlaveStm32f1{
    pub fn new() ->Self{
        Self{
            buffer : Vec::<u8, 255>::new(),
            slave_state : DpSlaveState::Por,
            stream_state : StreamState::WaitSyn,
        }
    }
}