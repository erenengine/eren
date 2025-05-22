import { HemisphericLight, Scene, Vector3 } from '@babylonjs/core';
import Light from './Light.js';

export default class HemisphereLight extends Light {
  private direction: Vector3;

  constructor(x: number, y: number, z: number) {
    super();
    this.direction = new Vector3(x, y, z);
  }

  protected createLight(scene: Scene) {
    return new HemisphericLight('', this.direction, scene);
  }
}