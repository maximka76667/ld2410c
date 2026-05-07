#![no_std]

const FRAME_HEADER: [u8; 4] = [0xF4, 0xF3, 0xF2, 0xF1];

pub trait UartReader {
    type Error;
    // We don't need Send here, we are working with 1-thread MCUs anyway
    fn read_until_idle(
        &mut self,
        buf: &mut [u8],
    ) -> impl Future<Output = Result<usize, Self::Error>>;
}

pub struct TargetData {
    pub status: u8,
    pub movement_distance: u16,
    pub movement_energy: u8,
    pub stationary_distance: u16,
    pub stationary_energy: u8,
    pub detection_distance: u16,
}

pub struct Ld2410c<U: UartReader> {
    uart: U,
}

impl<U: UartReader> Ld2410c<U> {
    pub fn new(uart: U) -> Self {
        Self { uart }
    }

    pub async fn read_frame(&mut self, buf: &mut [u8]) -> Result<Option<TargetData>, U::Error> {
        let n = self.uart.read_until_idle(buf).await?;
        Ok(parse_frame(&buf[..n]))
    }
}

fn parse_frame(data: &[u8]) -> Option<TargetData> {
    if data.len() < 17 || data[0..4] != FRAME_HEADER {
        return None;
    }

    Some(TargetData {
        status: data[8],
        movement_distance: u16::from_le_bytes([data[9], data[10]]),
        movement_energy: data[11],
        stationary_distance: u16::from_le_bytes([data[12], data[13]]),
        stationary_energy: data[14],
        detection_distance: u16::from_le_bytes([data[15], data[16]]),
    })
}
