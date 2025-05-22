import { TransformNode } from '@babylonjs/core';
import GameObject from './GameObject.js';

export default class DisplayNode {
  constructor(private container: TransformNode) { }

  addTo(parent: GameObject): this {
    parent.container.addChild(this.container);
    return this;
  }
}
