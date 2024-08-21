use zbase32::encode;

pub struct Registry;

impl Registry {
    pub fn build_uri(arr: &[u8]) -> String {
        let mut full_key = [0u8; 34];
        full_key[..32].copy_from_slice(&arr[..32]);
        let crc = CRC14::compute(&full_key[..32]);
        full_key[32] = (crc & 0xff) as u8;
        full_key[33] = ((crc & 0xff00) >> 8) as u8;
        format!("rho:id:{}", encode(&full_key, 270))
    }
}

struct CRC14;

impl CRC14 {
    const INIT_REMAINDER: u16 = 0;

    fn update(rem: u16, b: u8) -> u16 {
        let mut rem = rem ^ ((b as u16) << 6);

        for _ in 0..8 {
            let shift_rem = rem << 1;
            if (shift_rem & 0x4000) != 0 {
                rem = (shift_rem ^ 0x4805) as u16;
            } else {
                rem = shift_rem as u16;
            }
        }

        rem
    }

    pub fn compute(b: &[u8]) -> u16 {
        b.iter()
            .fold(Self::INIT_REMAINDER, |rem, &byte| Self::update(rem, byte))
    }
}
