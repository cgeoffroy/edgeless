// SPDX-FileCopyrightText: © 2023 Technical University of Munich, Chair of Connected Mobility
// SPDX-FileCopyrightText: © 2023 Siemens AG
// SPDX-License-Identifier: MIT

#[derive(Clone, minicbor::Decode, minicbor::Encode, minicbor::CborLen)]
pub enum EventData<T> {
    #[n(0)]
    Call(#[n(0)] T),
    #[n(1)]
    Cast(#[n(0)] T),
    #[n(2)]
    CallRet(#[n(0)] T),
    #[n(3)]
    CallNoRet,
    #[n(4)]
    Err,
}

//#[derive(Clone, Debug, PartialEq)]
//pub struct InvocationTraceId(u128);

#[derive(Clone, Debug, PartialEq, minicbor::Decode, minicbor::Encode, minicbor::CborLen)]
pub struct EventMetadata {
    #[n(0)]
    pub trace_id: [u8; 16],
    //pub trace_id: InvocationTraceId,
    // pub trace_id_low: u64,
    // #[n(1)]
    // pub trace_id_high: u64,
    // #[n(2)]
    #[n(1)]
    pub span_id: u64,
}

impl EventMetadata {
    pub fn new(trace_id: &[u8; 16], span_id: u64) -> Self {
        return Self {
            trace_id: trace_id.clone(),
            // trace_id: InvocationTraceId(trace_id),
            //trace_id_high: (trace_id >> 64) as u64,
            //trace_id_low: trace_id as u64,
            span_id,
        };
    }

    pub fn from(trace_id: u128, span_id: u64) -> Self {
        return Self::new(&Self::convert(trace_id), span_id);
    }

    pub fn convert(trace_id: u128) -> [u8; 16] {
        let a = opentelemetry::trace::TraceId::from(trace_id);
        a.to_bytes()
    }

    pub fn to_words(&self) -> [u64; 2] {
        let tmp = u128::from_be_bytes(self.trace_id);
        //let words: [u64; 2] = unsafe { std::mem::transmute::<u128, [u64; 2]>(tmp) };
        let words: [u64; 2] = [tmp as u64, (tmp >> 64) as u64];
        return words;
    }

    pub fn into_ctx(&self) -> opentelemetry::trace::SpanContext {
        let t = opentelemetry::trace::TraceId::from_bytes(self.trace_id);
        let s = opentelemetry::trace::SpanId::from_u64(self.span_id);
        let tf = opentelemetry::trace::TraceFlags::SAMPLED;
        let ts = opentelemetry::trace::TraceState::NONE;
        return opentelemetry::trace::SpanContext::new(t, s, tf, true, ts);
    }
}

// impl<'b, C> minicbor::Decode<'b, C> for InvocationTraceId {
//     fn decode(d: &mut minicbor::Decoder<'b>, _: &mut C) -> Result<Self, minicbor::decode::Error> {
//         let raw: &[u8] = d.bytes()?;
//         if raw.len() != 16 {
//             return Err(minicbor::decode::Error::message("Invalide size"));
//         }
//         let tmp: &[u8; 16] = raw.try_into().unwrap();
//         Ok(InvocationTraceId(u128::from_be_bytes(tmp.clone())))
//     }

//     fn nil() -> Option<Self> {
//         Some(InvocationTraceId(0))
//     }
// }

// impl<C> minicbor::Encode<C> for InvocationTraceId {
//     fn encode<W: minicbor::encode::Write>(&self, e: &mut minicbor::Encoder<W>, _: &mut C) -> Result<(), minicbor::encode::Error<W::Error>> {
//         let tmp: [u8; 16] = self.0.to_be_bytes();
//         e.bytes(&tmp).map(|_| ())
//     }
// }

// impl<C> minicbor::CborLen<C> for InvocationTraceId {
//     fn cbor_len(&self, _: &mut C) -> usize {
//         16 + 1 // 16 for the u128 and 1 for the array size
//     }
// }

mod test {
    #[test]
    fn size_matches() {
        let mut buffer = [0 as u8; 32];

        //let id = super::InvocationTraceId(1);
        let id = super::EventMetadata::from(1234, 45678);

        minicbor::encode(id.clone(), &mut buffer[..]).unwrap();

        let len = minicbor::len(id.clone());

        //let id2: super::InvocationTraceId = minicbor::decode(&buffer[..len]).unwrap();
        let id2: super::EventMetadata = minicbor::decode(&buffer[..len]).unwrap();

        assert_eq!(id, id2);
    }
}

#[derive(Clone, minicbor::Decode, minicbor::Encode, minicbor::CborLen)]
pub struct Event<T> {
    #[n(0)]
    pub target: crate::instance_id::InstanceId,
    #[n(1)]
    pub source: crate::instance_id::InstanceId,
    #[n(2)]
    pub stream_id: u64,
    #[n(3)]
    pub metadata: Option<EventMetadata>,
    #[n(34)]
    pub data: EventData<T>,
    #[n(4)]
    pub created: crate::event_timestamp::EventTimestamp,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LinkProcessingResult {
    FINAL,
    PROCESSED,
    PASSED,
}
