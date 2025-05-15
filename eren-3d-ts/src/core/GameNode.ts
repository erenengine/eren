import * as BABYLON from '@babylonjs/core';
import * as PIXI from 'pixi.js';
import GameSettings from './GameSettings';

export default class GameNode {
  private node2D: PIXI.Container | undefined;
  private node3D: BABYLON.TransformNode | undefined;

  constructor() {
    if (GameSettings['2d']) {
      this.node2D = new PIXI.Container();
    } else {
      this.node3D = new BABYLON.TransformNode('');
    }
  }
}
