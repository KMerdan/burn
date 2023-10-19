use burn_common::benchmark::BenchmarkResult;

use crate::{
    server::{ComputeServer, Handle},
    tune::{AutotuneOperation, MutBenchmark, Operation, TuneBenchmark, Tuner},
};

/// Server with extra capability of autotuning kernels
#[derive(Debug)]
pub(crate) struct AutotuneServer<S> {
    pub server: S,
    pub tuner: Tuner<S>,
}

impl<S: ComputeServer> AutotuneServer<S> {
    pub fn new(server: S) -> Self {
        AutotuneServer {
            server,
            tuner: Tuner::new(),
        }
    }

    pub fn execute_autotune(
        &mut self,
        autotune_operation: Box<dyn AutotuneOperation<S>>,
        execution_handles: &[&Handle<S>],
    ) {
        let mut cache_result = self.tuner.try_cache(&autotune_operation);
        if cache_result.is_none() {
            let autotune_handles: Vec<Handle<S>> = autotune_operation
                .inputs()
                .iter()
                .map(|input| self.server.create(input))
                .collect();
            let results = autotune_operation
                .autotunables()
                .into_iter()
                .map(|op| self.run_benchmark(op, autotune_handles.clone()))
                .collect();
            let fastest_index = self.tuner.find_fastest(results);
            self.tuner
                .cache_insert(autotune_operation.key(), fastest_index);
            cache_result = self.tuner.try_cache(&autotune_operation);
        }
        let operation = cache_result.unwrap();
        let kernel = operation.get_kernel(); // not sure
        self.server.execute_kernel(kernel, execution_handles);
    }

    fn run_benchmark(
        &mut self,
        operation: Operation<S>,
        handles: Vec<Handle<S>>,
    ) -> BenchmarkResult {
        TuneBenchmark::new(operation, handles, &mut self.server).run()
    }
}
