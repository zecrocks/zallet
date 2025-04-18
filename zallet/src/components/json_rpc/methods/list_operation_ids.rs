use jsonrpsee::core::RpcResult;

use crate::components::json_rpc::asyncop::{AsyncOperation, OperationState};

/// Response to a `z_listoperationids` RPC request.
pub(crate) type Response = RpcResult<Vec<String>>;

pub(crate) async fn call(async_ops: &[AsyncOperation], status: Option<&str>) -> Response {
    // - The outer `Option` indicates whether or not we are filtering.
    // - The inner `Option` indicates whether or not we recognise the requested state
    //   (`zcashd` treats unrecognised state strings as non-matching).
    let state = status.map(OperationState::parse);

    let mut operation_ids = vec![];

    for op in async_ops {
        match state {
            None => operation_ids.push(op.operation_id().into()),
            Some(f) => {
                let op_state = op.state().await;
                if f == Some(op_state) {
                    operation_ids.push(op.operation_id().into());
                }
            }
        }
    }

    Ok(operation_ids)
}
