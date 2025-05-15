import * as BABYLON from '@babylonjs/core';
import * as PIXI from 'pixi.js';

export default class Sprite {
  private sprite2D: PIXI.Sprite | undefined;
  private sprite3D: BABYLON.Sprite | undefined;

  constructor() {
  }
}