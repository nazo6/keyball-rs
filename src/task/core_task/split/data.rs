use rkyv::ser::serializers::BufferSerializer;
use rkyv::ser::Serializer;
use rkyv::{AlignedBytes, Archive, Deserialize, Serialize};

pub const MAX_DATA_SIZE: usize = 20;

macro_rules! impl_serde {
    ($s:ident) => {
        impl $s {
            pub fn to_bytes(&self) -> AlignedBytes<MAX_DATA_SIZE> {
                let mut serializer =
                    BufferSerializer::with_pos(AlignedBytes([0u8; MAX_DATA_SIZE]), 0);
                serializer.serialize_value(self).unwrap();
                serializer.into_inner()
            }

            // TODO: 入力値チェック
            pub fn from_bytes(data: &[u8]) -> Self {
                let archived = unsafe { rkyv::archived_value::<Self>(data, 0) };
                archived.deserialize(&mut rkyv::Infallible).unwrap()
            }
        }
    };
}

#[derive(Archive, Deserialize, Serialize, Debug)]
// #[archive(check_bytes)]
pub enum MasterToSlave {
    Led,
    Message(u8),
}

impl_serde!(MasterToSlave);

#[derive(Archive, Deserialize, Serialize, Debug)]
// #[archive(check_bytes)]
pub enum SlaveToMaster {
    Pressed { keys: [Option<(u8, u8)>; 6] },
    Mouse { x: i8, y: i8 },
    Message(u8),
}

impl_serde!(SlaveToMaster);

#[derive(Archive, Deserialize, Serialize, Debug)]
// #[archive(check_bytes)]
pub enum KeyChangeType {
    Pressed,
    Released,
}
