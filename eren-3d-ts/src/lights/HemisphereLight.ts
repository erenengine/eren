import { HemisphericLight, Vector3 } from '@babylonjs/core';
import Light from './Light.js';

export default class HemisphereLight extends Light {
  constructor(x: number, y: number, z: number) {
    super(new HemisphericLight('', new Vector3(x, y, z)));
  }
}