import { TransformNode } from '@babylonjs/core';
import DisplayNode from './DisplayNode.js';

export default class GameObject extends DisplayNode {
  constructor(x: number, y: number, z: number) {
    const transformNode = new TransformNode('');
    transformNode.position.set(x, y, z);
    super(transformNode);
  }
}
