import onnx
from onnx import helper, TensorProto
import numpy as np

# Create input and output
X = helper.make_tensor_value_info('input', TensorProto.FLOAT, [1])
Y = helper.make_tensor_value_info('output', TensorProto.FLOAT, [1])

# Create an identity node (output = input)
identity_node = helper.make_node(
    'Identity',
    inputs=['input'],
    outputs=['output']
)

# Create the graph
graph_def = helper.make_graph(
    [identity_node],
    'identity_model',
    [X],
    [Y]
)

# Create the model with IR version 8 (compatible with ort 1.16)
model_def = helper.make_model(graph_def, producer_name='onnx-example')
model_def.ir_version = 8  # Set compatible IR version
model_def.opset_import[0].version = 13  # Use opset 13

# Save the model
onnx.save(model_def, 'identity_v8.onnx')
print("Created identity_v8.onnx with IR version 8")