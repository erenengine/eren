import { TransformNode } from '@babylonjs/core';
import AssetLoader from '../assets/AssetLoader.js';
import DisplayNode from '../core/DisplayNode.js';

export default class Model extends DisplayNode {
  constructor(x: number, y: number, z: number, assetId: string) {
    const modelData = AssetLoader.getModelData(assetId);
    const transformNode = new TransformNode('');
    transformNode.position.set(x, y, z);
    for (const mesh of modelData.meshes) {
      mesh.parent = transformNode;
    }
    super(transformNode);
  }
}