// SPDX-FileCopyrightText: © 2024 Technical University of Munich, Chair of Connected Mobility
// SPDX-FileCopyrightText: © 2024 Claudio Cicconetti <c.cicconetti@iit.cnr.it>
// SPDX-FileCopyrightText: © 2024 Siemens AG
// SPDX-License-Identifier: MIT

use std::borrow::BorrowMut;

use wasmtime::AsContextMut;

use opentelemetry::{
    global,
    trace::{Span, SpanKind, Status, TraceContextExt, Tracer},
    KeyValue,
};

/// Binds the WASM component's imports to the function's GuestAPIHost.
pub struct GuestAPI {
    pub host: crate::base_runtime::guest_api::GuestAPIHost,
}

pub async fn telemetry_log(
    mut caller: wasmtime::Caller<'_, GuestAPI>,
    level: i32,
    target_ptr: i32,
    target_len: i32,
    msg_ptr: i32,
    msg_len: i32,
) -> wasmtime::Result<()> {
    let mem = get_memory(&mut caller)?;
    let target = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, target_ptr, target_len)?;
    let msg = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, msg_ptr, msg_len)?;

    caller
        .data_mut()
        .host
        .telemetry_log(super::helpers::level_from_i32(level), &target, &msg)
        .await;
    Ok(())
}

pub async fn cast_raw(
    mut caller: wasmtime::Caller<'_, GuestAPI>,
    instance_node_id_ptr: i32,
    instance_component_id_ptr: i32,
    payload_ptr: i32,
    payload_len: i32,
) -> wasmtime::Result<()> {
    log::error!("cast_raw");
    let mem = get_memory(&mut caller)?;
    let node_id = mem.data_mut(&mut caller)[instance_node_id_ptr as usize..(instance_node_id_ptr as usize) + 16_usize].to_vec();
    let component_id = mem.data_mut(&mut caller)[instance_component_id_ptr as usize..(instance_component_id_ptr as usize) + 16_usize].to_vec();
    let instance_id = edgeless_api::function_instance::InstanceId {
        node_id: uuid::Uuid::from_bytes(node_id.try_into().map_err(|_| wasmtime::Error::msg("uuid error"))?),
        function_id: uuid::Uuid::from_bytes(component_id.try_into().map_err(|_| wasmtime::Error::msg("uuid error"))?),
    };
    let payload = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, payload_ptr, payload_len)?;
    let this_metadata = edgeless_api_core::invocation::EventMetadata::from(4242, 4242);

    let u = caller.get_export("get_metadata_span_id");
    let y = u.unwrap().into_func();
    log::error!("Cast raw {:?}", y);
    println!("Cast raw {:?}", y);
    let mut x = [wasmtime::Val::I64(4)];
    y.unwrap()
        .call_async(&mut caller, &[], &mut x)
        .await
        .map_err(|e| wasmtime::Error::msg(format!("get_metadata {:?}", e)))?;

    caller
        .data_mut()
        .host
        .cast_raw(instance_id, Some(&this_metadata), &payload)
        .await
        .map_err(|_| wasmtime::Error::msg("string error"))?;
    Ok(())
}

pub async fn call_raw(
    mut caller: wasmtime::Caller<'_, GuestAPI>,
    instance_node_id_ptr: i32,
    instance_component_id_ptr: i32,
    payload_ptr: i32,
    payload_len: i32,
    out_ptr_ptr: i32,
    out_len_ptr: i32,
) -> wasmtime::Result<i32> {
    log::error!("call_raw");
    let mem = get_memory(&mut caller)?;
    let alloc = get_alloc(&mut caller)?;
    let node_id = mem.data_mut(&mut caller)[instance_node_id_ptr as usize..(instance_node_id_ptr as usize) + 16_usize].to_vec();
    let component_id = mem.data_mut(&mut caller)[instance_component_id_ptr as usize..(instance_component_id_ptr as usize) + 16_usize].to_vec();
    let instance_id = edgeless_api::function_instance::InstanceId {
        node_id: uuid::Uuid::from_bytes(node_id.try_into().map_err(|_| wasmtime::Error::msg("uuid error"))?),
        function_id: uuid::Uuid::from_bytes(component_id.try_into().map_err(|_| wasmtime::Error::msg("uuid error"))?),
    };
    let payload = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, payload_ptr, payload_len)?;
    let this_metadata = edgeless_api_core::invocation::EventMetadata::from(4242, 4242);

    let call_ret = caller
        .data_mut()
        .host
        .call_raw(instance_id, Some(&this_metadata), &payload)
        .await
        .map_err(|_| wasmtime::Error::msg("call error"))?;
    match call_ret {
        edgeless_dataplane::core::CallRet::NoReply => Ok(0),
        edgeless_dataplane::core::CallRet::Reply(data) => {
            let len = data.len();

            let data_ptr = super::helpers::copy_to_vm(&mut caller.as_context_mut(), &mem, &alloc, data.as_bytes()).await?;
            super::helpers::copy_to_vm_ptr(&mut caller.as_context_mut(), &mem, out_ptr_ptr, &data_ptr.to_le_bytes())?;
            super::helpers::copy_to_vm_ptr(&mut caller.as_context_mut(), &mem, out_len_ptr, &len.to_le_bytes())?;

            Ok(1)
        }
        edgeless_dataplane::core::CallRet::Err => Ok(2),
    }
}

async fn gg(caller: &mut wasmtime::Caller<'_, GuestAPI>) -> wasmtime::Result<(u128, u64)> {
    let gmsi = caller.borrow_mut().get_export("get_metadata_span_id").unwrap().into_func();
    log::warn!("gmsi {:?}", gmsi);
    let mut x = [wasmtime::Val::I64(4564)];
    log::error!("_______________________________ {:?}", x[0]);
    gmsi.unwrap()
        .call_async(&mut caller.borrow_mut(), &[], &mut x)
        .await
        .map_err(|e| wasmtime::Error::msg(format!("get_metadata {:?}", e)))?;
    log::error!("_______________________________ pre span_id {:?}", x[0]);
    let span_id = x[0].clone().unwrap_i64() as u64;
    log::error!("_______________________________ span_id {:?}", span_id);

    let gmtdl = caller.borrow_mut().get_export("get_metadata_trace_id_low").unwrap().into_func();
    log::warn!("gmtdl {:?}", gmtdl);
    log::error!("_______________________________ {:?}", x[0]);
    gmtdl
        .unwrap()
        .call_async(&mut caller.borrow_mut(), &[], &mut x)
        .await
        .map_err(|e| wasmtime::Error::msg(format!("get_metadata_trace_id_low {:?}", e)))?;
    log::error!("_______________________________ pre trace_id_low {:?}", x[0]);
    let trace_id_low = x[0].clone().unwrap_i64() as u64;
    log::error!("_______________________________ trace_id_low {:?}", trace_id_low);

    let gmtdh = caller.borrow_mut().get_export("get_metadata_trace_id_high").unwrap().into_func();
    log::warn!("gmtdh {:?}", gmtdh);
    log::error!("_______________________________ {:?}", x[0]);
    gmtdh
        .unwrap()
        .call_async(&mut caller.borrow_mut(), &[], &mut x)
        .await
        .map_err(|e| wasmtime::Error::msg(format!("get_metadata_trace_id_high {:?}", e)))?;
    log::error!("_______________________________ pre trace_id_high {:?}", x[0]);
    let trace_id_high = x[0].clone().unwrap_i64() as u64;
    log::error!("_______________________________ trace_id_high {:?}", trace_id_high);

    let res: u128 = unsafe { std::mem::transmute::<[u64; 2], u128>([trace_id_low, trace_id_high]) };
    log::error!("_______________________________ res {:?}", res);

    Ok((res, span_id))
}

pub async fn cast(
    mut caller: wasmtime::Caller<'_, GuestAPI>,
    target_ptr: i32,
    target_len: i32,
    payload_ptr: i32,
    payload_len: i32,
) -> wasmtime::Result<()> {
    let mem = get_memory(&mut caller)?; // HERE

    // let u: Option<wasmtime::Extern> = caller.get_export("get_metadata_span_id");
    // let y = u.unwrap().into_func();
    // log::error!("Cast raw {:?}", y);
    // println!("Cast raw {:?}", y);
    // let mut x = [wasmtime::Val::I64(4564)];
    // log::error!("_______________________________ {:?}", x[0]);

    // y.unwrap()
    //     .call_async(&mut caller, &[], &mut x)
    //     .await
    //     .map_err(|e| wasmtime::Error::msg(format!("get_metadata {:?}", e)))?;
    // log::error!("_______________________________ {:?}", x[0]);
    // let span_id = x[0].clone().unwrap_i64() as u64;

    // y.unwrap()
    //     .call_async(&mut caller, &[], &mut x)
    //     .await
    //     .map_err(|e| wasmtime::Error::msg(format!("get_metadata {:?}", e)))?;
    // log::error!("_______________________________ {:?}", x[0]);
    let (trace_id, span_id) = gg(&mut caller).await?;
    let this_metadata = edgeless_api_core::invocation::EventMetadata::from(trace_id, span_id);

    let otelctx = this_metadata.into_ctx();

    let tracer = opentelemetry::global::tracer("scope-node");
    let parent_cx = opentelemetry::Context::current().with_remote_span_context(otelctx);
    let mut span = tracer
        .span_builder("guest_api_binding:cast")
        .with_kind(SpanKind::Internal)
        .start_with_context(&tracer, &parent_cx);

    let target = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, target_ptr, target_len)?;
    let payload = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, payload_ptr, payload_len)?;

    match caller.data_mut().host.cast_alias(&target, Some(&this_metadata), &payload).await {
        Ok(_) => {}
        Err(_) => {
            // We ignore casts to unknown targets.
            log::warn!("Cast to unknown target: {}", target);
        }
    };

    span.set_attribute(KeyValue::new("target", target));
    span.set_status(opentelemetry::trace::Status::Ok);
    span.end();

    Ok(())
}

pub async fn call(
    mut caller: wasmtime::Caller<'_, GuestAPI>,
    target_ptr: i32,
    target_len: i32,
    payload_ptr: i32,
    payload_len: i32,
    out_ptr_ptr: i32,
    out_len_ptr: i32,
) -> wasmtime::Result<i32> {
    let mem = get_memory(&mut caller)?;
    let alloc = get_alloc(&mut caller)?;

    let target = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, target_ptr, target_len)?;
    let payload = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, payload_ptr, payload_len)?;

    log::info!("Call {} {}", target, payload);
    let this_metadata = edgeless_api_core::invocation::EventMetadata::from(4242, 4242);

    let call_ret = caller
        .data_mut()
        .host
        .call_alias(&target, Some(&this_metadata), &payload)
        .await
        .map_err(|_| wasmtime::Error::msg("call error"))?;
    match call_ret {
        edgeless_dataplane::core::CallRet::NoReply => Ok(0),
        edgeless_dataplane::core::CallRet::Reply(data) => {
            let len = data.len();

            let data_ptr = super::helpers::copy_to_vm(&mut caller.as_context_mut(), &mem, &alloc, data.as_bytes()).await?;
            super::helpers::copy_to_vm_ptr(&mut caller.as_context_mut(), &mem, out_ptr_ptr, &data_ptr.to_le_bytes())?;
            super::helpers::copy_to_vm_ptr(&mut caller.as_context_mut(), &mem, out_len_ptr, &len.to_le_bytes())?;

            Ok(1)
        }
        edgeless_dataplane::core::CallRet::Err => Ok(2),
    }
}

pub async fn delayed_cast(
    mut caller: wasmtime::Caller<'_, GuestAPI>,
    delay_ms: i64,
    target_ptr: i32,
    target_len: i32,
    payload_ptr: i32,
    payload_len: i32,
) -> wasmtime::Result<()> {
    let mem = get_memory(&mut caller)?;

    let (trace_id, span_id) = gg(&mut caller).await?;
    let this_metadata = edgeless_api_core::invocation::EventMetadata::from(trace_id, span_id);

    let otelctx = this_metadata.into_ctx();

    let tracer = opentelemetry::global::tracer("scope-node");
    let parent_cx = opentelemetry::Context::current().with_remote_span_context(otelctx);
    let mut span = tracer
        .span_builder("guest_api_binding:delayed_cast")
        .with_kind(SpanKind::Server)
        .start_with_context(&tracer, &parent_cx);

    let target = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, target_ptr, target_len)?;
    let payload = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, payload_ptr, payload_len)?;

    //let this_metadata = edgeless_api_core::invocation::EventMetadata::from(4242, 4242);

    caller
        .data_mut()
        .host
        .delayed_cast(delay_ms as u64, &target, Some(&this_metadata), &payload)
        .await
        .map_err(|_| wasmtime::Error::msg("call error"))?;

    span.set_status(opentelemetry::trace::Status::Ok);
    span.end();

    Ok(())
}

pub async fn sync(mut caller: wasmtime::Caller<'_, GuestAPI>, state_ptr: i32, state_len: i32) -> wasmtime::Result<()> {
    let mem = get_memory(&mut caller)?;
    let state = super::helpers::load_string_from_vm(&mut caller.as_context_mut(), &mem, state_ptr, state_len)?;

    caller
        .data_mut()
        .host
        .sync(&state)
        .await
        .map_err(|_| wasmtime::Error::msg("sync error"))?;
    Ok(())
}

pub async fn slf(mut caller: wasmtime::Caller<'_, GuestAPI>, out_node_id_ptr: i32, out_component_id_ptr: i32) -> wasmtime::Result<()> {
    let mem = get_memory(&mut caller)?;

    let id = caller.data_mut().host.slf().await;

    super::helpers::copy_to_vm_ptr(&mut caller.as_context_mut(), &mem, out_node_id_ptr, id.node_id.as_bytes())?;
    super::helpers::copy_to_vm_ptr(&mut caller.as_context_mut(), &mem, out_component_id_ptr, id.function_id.as_bytes())?;

    Ok(())
}

pub(crate) fn get_memory(caller: &mut wasmtime::Caller<'_, super::guest_api_binding::GuestAPI>) -> wasmtime::Result<wasmtime::Memory> {
    caller
        .get_export("memory")
        .ok_or(wasmtime::Error::msg("memory error"))?
        .into_memory()
        .ok_or(wasmtime::Error::msg("memory error"))
}

pub(crate) fn get_alloc(caller: &mut wasmtime::Caller<'_, super::guest_api_binding::GuestAPI>) -> wasmtime::Result<wasmtime::TypedFunc<i32, i32>> {
    caller
        .get_export("edgeless_mem_alloc")
        .ok_or(wasmtime::Error::msg("alloc error"))?
        .into_func()
        .ok_or(wasmtime::Error::msg("alloc error"))?
        .typed::<i32, i32>(&caller)
        .map_err(|_| wasmtime::Error::msg("alloc error"))
}
