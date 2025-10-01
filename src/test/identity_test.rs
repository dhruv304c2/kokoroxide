use std::sync::Arc;
use ndarray::{Array1, CowArray, IxDyn};
use ort::{Environment, GraphOptimizationLevel, SessionBuilder, Value};

pub fn run_identity() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the ONNX Runtime environment
    let env = Arc::new(Environment::builder().with_name("id").build()?);

    // Build the session
    let session = SessionBuilder::new(&env)?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_model_from_file("models/identity_v8.onnx")?;

    // Create input tensor
    let input = Array1::<f32>::from_vec(vec![2.0]);
    let input_cow: CowArray<f32, IxDyn> = CowArray::from(input.into_dyn());
    let input_tensor = Value::from_array(session.allocator(), &input_cow)?;

    // Run inference
    let outputs = session.run(vec![input_tensor])?;
    let output = outputs[0].try_extract::<f32>()?;
    let output_view = output.view();
    println!("out = {:?}", output_view.as_slice().unwrap());
    Ok(())
}
