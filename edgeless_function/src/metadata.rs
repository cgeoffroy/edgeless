// SPDX-License-Identifier: MIT

// use opentelemetry::trace::{SpanId, TraceId};

use core::fmt::Binary;

#[derive(Clone)]
pub struct Metadata {
    pub trace_id: u128,
    pub span_id: u64,
    pub parent_id: u64,
}

static mut CURRENT_METADATA: Metadata = Metadata {
    trace_id: 0u128,
    span_id: 0u64,
    parent_id: 0u64,
};

#[no_mangle]
pub unsafe extern "C" fn get_metadata_trace_id_high() -> u64 {
    return (CURRENT_METADATA.trace_id >> 64) as u64;
}

#[no_mangle]
pub unsafe extern "C" fn get_metadata_trace_id_low() -> u64 {
    return CURRENT_METADATA.trace_id as u64;
}

#[no_mangle]
pub unsafe extern "C" fn get_metadata_span_id() -> u64 {
    return CURRENT_METADATA.span_id;
}

#[no_mangle]
pub unsafe extern "C" fn get_metadata_parent_id() -> u64 {
    return CURRENT_METADATA.parent_id;
}

#[no_mangle]
pub unsafe extern "C" fn set_metadata(trace_id_low: u64, trace_id_high: u64, span_id: u64, parent_id: u64) {
    log::warn!(
        "set_metadata {:?} with {:?},{:?},{:?},{:?}",
        CURRENT_METADATA.span_id,
        trace_id_low,
        trace_id_high,
        span_id,
        parent_id
    );
    CURRENT_METADATA.trace_id = std::mem::transmute::<[u64; 2], u128>([trace_id_low, trace_id_high]);
    //CURRENT_METADATA.trace_id = std::mem::transmute::<(u64, u64), u128>((trace_id_low, trace_id_high));
    CURRENT_METADATA.span_id = span_id;
    CURRENT_METADATA.parent_id = parent_id;
    log::warn!("gset_metadata {:?}", CURRENT_METADATA.span_id);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_f() {
        let words: [u64; 2] = unsafe { std::mem::transmute::<u128, [u64; 2]>(1u128) };
        assert_eq!(1, words[0]);
        assert_eq!(0, words[1]);
        let _ = unsafe {
            set_metadata(1, 0, 99, 99);
        };
        assert_eq!(1, unsafe { CURRENT_METADATA.trace_id });
        let _ = unsafe {
            set_metadata(0, 1, 99, 99);
        };
        assert_eq!(1u128 << 64, unsafe { CURRENT_METADATA.trace_id });
        let words: [u64; 2] = unsafe { std::mem::transmute::<u128, [u64; 2]>(1234u128) };
        let _ = unsafe {
            set_metadata(words[0], words[1], 99, 99);
        };
        assert_eq!(1234u128, unsafe { CURRENT_METADATA.trace_id });
    }

    #[test]
    fn test_g() {
        let _ = unsafe {
            set_metadata(12, 0, 45, 67);
        };
        assert_eq!(12, unsafe { get_metadata_trace_id_high() });
        assert_eq!(23, unsafe { get_metadata_trace_id_low() });
        assert_eq!(45, unsafe { get_metadata_span_id() });
        assert_eq!(67, unsafe { get_metadata_parent_id() });
    }
}
