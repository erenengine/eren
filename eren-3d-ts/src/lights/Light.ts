import { Light as BabylonLight, Scene } from '@babylonjs/core';

export default abstract class Light {
  protected abstract createLight(scene: Scene): BabylonLight;

  setScene(scene: Scene) {
    this.createLight(scene);
  }
}