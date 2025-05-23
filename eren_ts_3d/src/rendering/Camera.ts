import { ArcRotateCamera, Scene, UniversalCamera, Vector3 } from '@babylonjs/core';

export default class Camera {
  private cameraPosition: Vector3;
  private universalCamera?: UniversalCamera;

  constructor(x: number, y: number, z: number) {
    this.cameraPosition = new Vector3(x, y, z);
  }

  setScene(scene: Scene) {
    this.universalCamera = new UniversalCamera('', this.cameraPosition, scene);
    this.universalCamera.setTarget(Vector3.Zero());
    scene.activeCamera = this.universalCamera;
  }
}