import * as BABYLON from '@babylonjs/core';
import * as PIXI from 'pixi.js';
import GameSettings from './GameSettings';

export default class Renderer {
  private renderer2D: PIXI.Renderer | undefined;
  private renderer3D: BABYLON.Engine | undefined;

  constructor(private canvas: HTMLCanvasElement) {
    this.init();
  }

  private async init() {
    if (GameSettings['2d']) {
      this.renderer2D = await PIXI.autoDetectRenderer({
        view: this.canvas
      });
    } else {
      this.renderer3D = new BABYLON.Engine(this.canvas, true);
    }
  }
}
