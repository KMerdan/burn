use burn_tensor::{ops::FloatElem, Data, Reader};

use crate::{
    graph::{FusedBackend, GraphExecution, TensorOps},
    FusionServer, FusionTensor, TensorDefinition,
};

pub trait FusionClient: Send + Sync + Clone + core::fmt::Debug {
    type FusedBackend: FusedBackend;
    type GraphExecution: GraphExecution<Self::FusedBackend>;

    fn new(server: FusionServer<Self::FusedBackend, Self::GraphExecution>) -> Self;
    fn register(&self, ops: TensorOps<Self::FusedBackend>);
    fn sync(&self);
    fn empty(&self, shape: Vec<usize>) -> FusionTensor<Self>;
    fn device<'a>(&'a self) -> &'a <Self::FusedBackend as FusedBackend>::HandleDevice;
    fn read_float<const D: usize>(
        &self,
        tensor: TensorDefinition,
    ) -> Reader<Data<FloatElem<Self::FusedBackend>, D>>;
}
