use wasm_bindgen::JsError;

use crate::err::invalid_data_err;

const CLOCK_EPOCH_OFFSET: usize = 16;

pub(crate) fn try_clock_acc_data_epoch(d: &[u8]) -> Result<u64, JsError> {
    clock_acc_data_epoch(d).ok_or_else(invalid_data_err)
}

fn clock_acc_data_epoch(d: &[u8]) -> Option<u64> {
    d.split_at_checked(CLOCK_EPOCH_OFFSET)?
        .1
        .first_chunk()
        .map(|a| u64::from_le_bytes(*a))
}
