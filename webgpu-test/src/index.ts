async function webgpu() {
  if (!navigator.gpu) {
    console.error("WebGPU is not available.");
    throw new Error("WebGPU support is not available");
  }

  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) {
    console.error("Failed to request Adapter.");
    return;
  }

  let device = await adapter.requestDevice();
  if (!device) {
    console.error("Failed to request Device.");
    return;
  }

  const canvas = document.getElementById('canvas') as HTMLCanvasElement;
  const context = canvas.getContext('webgpu');
  if (!context) {
    console.error("Failed to get WebGPU context.");
    return;
  }

  const canvasConfig: GPUCanvasConfiguration = {
    device: device,
    format: navigator.gpu.getPreferredCanvasFormat(),
    usage:
      GPUTextureUsage.RENDER_ATTACHMENT,
    alphaMode: 'opaque'
  };

  context.configure(canvasConfig);

  let colorTexture = context.getCurrentTexture();
  let colorTextureView = colorTexture.createView();

  let colorAttachment: GPURenderPassColorAttachment = {
    view: colorTextureView,
    clearValue: { r: 1, g: 0, b: 0, a: 1 },
    loadOp: 'clear',
    storeOp: 'store'
  };

  const renderPassDesc: GPURenderPassDescriptor = {
    colorAttachments: [colorAttachment]
  };

  const commandEncoder = device.createCommandEncoder();

  const passEncoder = commandEncoder.beginRenderPass(renderPassDesc);
  passEncoder.setViewport(0, 0, canvas.width, canvas.height, 0, 1);
  passEncoder.end();

  device.queue.submit([commandEncoder.finish()]);
}
webgpu();