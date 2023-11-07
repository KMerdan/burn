use crate::{
    graph::{Graph, GraphExecution, Optimization, TensorOpsDescription},
    FusedBackend, FusionProperties, FusionStatus, FusionTensor, HandleContainer, TensorId,
};
use burn_tensor::ops::{FloatElem, IntElem};
use std::sync::Arc;

pub struct FusionServer<B, G>
where
    B: FusedBackend,
    G: GraphExecution<B>,
{
    optimizations: Vec<Optimization<B>>,
    graph: Graph<B>,
    handles: HandleContainer<B>,
    execution: G,
    pub device: B::HandleDevice,
}

/// Trait name graph execution strategy.
impl<B, G> FusionServer<B, G>
where
    B: FusedBackend,
    G: GraphExecution<B>,
{
    pub fn new(device: B::HandleDevice) -> Self {
        let optimizations = B::operations()
            .into_iter()
            .map(|ops| Optimization::new(ops, FusionStatus::Open(FusionProperties::default())))
            .collect();

        Self {
            optimizations,
            graph: Graph::new(),
            handles: HandleContainer::new(device.clone()),
            execution: G::default(),
            device,
        }
    }

    pub fn register(&mut self, ops: TensorOpsDescription<B>) {
        let ops = Arc::new(ops);
        self.graph.add(ops.clone());

        self.optimizations
            .iter_mut()
            .for_each(|optimization| optimization.register(&ops));

        self.execution.maybe_execute(
            &mut self.graph,
            &mut self.handles,
            &mut self.optimizations,
            false,
        );
    }

    pub fn sync(&mut self) {
        self.execution.maybe_execute(
            &mut self.graph,
            &mut self.handles,
            &mut self.optimizations,
            true,
        );
    }

    pub fn create_empty_handle(&mut self) -> Arc<TensorId> {
        self.handles.create_emtpy()
    }

    pub fn create_float_handle(&mut self, values: Vec<FloatElem<B>>) -> Arc<TensorId> {
        self.handles.create_float(values)
    }

    pub fn create_int_handle(&mut self, values: Vec<IntElem<B>>) -> Arc<TensorId> {
        self.handles.create_int(values)
    }

    pub fn create_bool_handle(&mut self, values: Vec<bool>) -> Arc<TensorId> {
        self.handles.create_bool(values)
    }

    pub fn read_float<const D: usize>(
        &mut self,
        tensor: crate::TensorDescription,
    ) -> burn_tensor::Reader<burn_tensor::Data<FloatElem<B>, D>> {
        // Make sure all registered operations are executed.
        // The underlying backend can still be async.
        self.sync();

        let tensor = self.handles.get_float_tensor(&tensor);
        B::into_data(tensor)
    }

    pub fn read_int<const D: usize>(
        &mut self,
        tensor: crate::TensorDescription,
    ) -> burn_tensor::Reader<burn_tensor::Data<IntElem<B>, D>> {
        // Make sure all registered operations are executed.
        // The underlying backend can still be async.
        self.sync();

        let tensor = self.handles.get_int_tensor(&tensor);
        B::int_into_data(tensor)
    }

    pub fn read_bool<const D: usize>(
        &mut self,
        tensor: crate::TensorDescription,
    ) -> burn_tensor::Reader<burn_tensor::Data<bool, D>> {
        // Make sure all registered operations are executed.
        // The underlying backend can still be async.
        self.sync();

        let tensor = self.handles.get_bool_tensor(&tensor);
        B::bool_into_data(tensor)
    }

    pub fn change_server<const D: usize>(
        &mut self,
        tensor: &crate::TensorDescription,
        device: &B::Device,
        server_device: &mut Self,
    ) -> Arc<TensorId> {
        let tensor = self.handles.get_float_tensor::<D>(&tensor);
        let tensor = B::to_device(tensor, &device);
        let id = server_device.create_empty_handle();

        server_device
            .handles
            .register_float_tensor(&id, tensor.clone());

        id
    }
}
